use std::{
    num::{NonZeroU32, NonZeroUsize},
    sync::Arc,
};

use alloy::{
    eips::BlockNumberOrTag,
    providers::Provider,
    rpc::types::Block,
    transports::{RpcError, TransportErrorKind},
};
use flume::Receiver;
use governor::{Quota, RateLimiter};
use tokio::sync::Semaphore;

use crate::indexer::provider::IndexerProvider;

pub struct BlockFetcherParams {
    pub max_concurrency: NonZeroUsize,
    pub max_rps: NonZeroU32,
    pub start_block: u64,
}

impl Default for BlockFetcherParams {
    fn default() -> Self {
        Self {
            max_concurrency: NonZeroUsize::new(100).unwrap(),
            max_rps: NonZeroU32::new(100).unwrap(),
            start_block: Default::default(),
        }
    }
}

pub struct BlockFetcher {
    rx: Receiver<Result<Option<Block>, RpcError<TransportErrorKind>>>,
}

impl BlockFetcher {
    pub async fn fetch(
        provider: IndexerProvider,
        params: BlockFetcherParams,
    ) -> Result<Self, crate::error::Error> {
        let (tx, rx) = flume::bounded(1000);

        let semaphore = Arc::new(Semaphore::new(params.max_concurrency.get() as usize));
        let rate_limiter = RateLimiter::direct(Quota::per_second(params.max_rps));
        let mut current_head = provider.current_head().clone();

        tokio::spawn(async move {
            let mut block_number = params.start_block;

            loop {
                let current_head_block_number = { current_head.borrow_and_update().number };

                if block_number >= current_head_block_number {
                    if let Err(_) = current_head.changed().await {
                        break;
                    }
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
                        .await;

                    tx.send_async(block).await.unwrap();

                    drop(permit)
                });

                block_number += 1;
            }
        });

        Ok(Self { rx })
    }

    pub async fn receiver(&self) -> &Receiver<Result<Option<Block>, RpcError<TransportErrorKind>>> {
        &self.rx
    }
}
