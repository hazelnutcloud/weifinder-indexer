use std::{num::NonZeroU32, sync::Arc, time::Duration};

use alloy::{
    eips::{BlockId, BlockNumberOrTag},
    providers::{Provider, ProviderBuilder, RootProvider, WsConnect},
    rpc::types::Block,
    transports::{RpcError, TransportErrorKind},
};
use diesel::{
    Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection,
    connection::SimpleConnection,
};
use governor::{Quota, RateLimiter};
use tokio::sync::{RwLock, Semaphore};
use tracing::error;

use crate::{models::Checkpoint, settings::Settings};

pub struct ChainIndexer {
    provider: RootProvider,
    chain_id: u64,
    current_head_block_number: Arc<RwLock<u64>>,
    db_conn: SqliteConnection,
    max_concurrency: usize,
    max_rps: NonZeroU32,
}

impl ChainIndexer {
    pub async fn from_settings(settings: &Settings) -> Result<Self, crate::error::Error> {
        let provider = ProviderBuilder::new()
            .disable_recommended_fillers()
            .connect_ws(WsConnect::new(&settings.rpc_url))
            .await?;

        let chain_id = provider.get_chain_id().await?;
        let current_head = provider
            .get_block(BlockId::latest())
            .await?
            .expect("latest block to exist");

        let mut db_conn = SqliteConnection::establish(&settings.db_url)?;
        db_conn.batch_execute("PRAGMA journal_mode = WAL;")?;

        Ok(Self {
            provider,
            chain_id,
            current_head_block_number: Arc::new(RwLock::new(current_head.number())),
            db_conn,
            max_concurrency: settings.fetcher_max_concurrency,
            max_rps: settings.fetcher_max_rps,
        })
    }

    pub async fn run(&mut self) -> Result<(), crate::error::Error> {
        use crate::schema::checkpoints::dsl::*;

        self.watch_chain_head().await?;

        let start_block: u64 = checkpoints
            .select(Checkpoint::as_select())
            .filter(chain_id.eq(self.chain_id as i32))
            .first(&mut self.db_conn)?
            .last_saved_block_number
            .map(|n| n as u64)
            .unwrap_or(0);

        Ok(())
    }

    async fn watch_chain_head(&self) -> Result<(), RpcError<TransportErrorKind>> {
        let mut sub = self.provider.subscribe_blocks().await?;
        let current_head_block_number = self.current_head_block_number.clone();

        tokio::spawn(async move {
            while let Ok(header) = sub.recv().await {
                let block_number = header.number;
                let mut n = current_head_block_number.write().await;
                *n = block_number;
            }

            error!("Chain head watcher exited");
        });

        Ok(())
    }

    async fn fetch_blocks(&self, start_block: u64) -> flume::Receiver<Block> {
        let (tx, rx) = flume::bounded(600);

        let provider = self.provider.clone();
        let semaphore = Arc::new(Semaphore::new(self.max_concurrency));
        let rate_limiter = RateLimiter::direct(Quota::per_second(self.max_rps));
        let current_head_block_number = self.current_head_block_number.clone();

        tokio::spawn(async move {
            let mut block_number = start_block;

            loop {
                let current_head_block_number = current_head_block_number.read().await;

                if block_number >= *current_head_block_number {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }

                let permit = semaphore
                    .clone()
                    .acquire_owned()
                    .await
                    .expect("semaphore to not be closed");
                rate_limiter.until_ready().await;

                let provider = provider.clone();
                let tx = tx.clone();

                tokio::spawn(async move {
                    let block = provider
                        .get_block_by_number(BlockNumberOrTag::Number(block_number))
                        .await
                        .unwrap()
                        .unwrap();

                    tx.send_async(block).await.unwrap();

                    drop(permit)
                });

                block_number += 1;
            }
        });

        rx
    }
}
