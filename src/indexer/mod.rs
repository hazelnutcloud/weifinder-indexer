mod block_fetcher;
mod head_watcher;
mod provider;

pub use block_fetcher::*;
pub use head_watcher::*;
pub use provider::*;

use std::num::NonZeroU32;

use alloy::{
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::client::ClientBuilder,
    transports::layers::RetryBackoffLayer,
};
use diesel::{
    Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper, SqliteConnection,
    connection::SimpleConnection,
};

use crate::{models::Checkpoint, settings::Settings};

pub struct ChainIndexer {
    provider: IndexerProvider,
    chain_id: u64,
    db_conn: SqliteConnection,
    block_fetcher: BlockFetcher,
}

impl ChainIndexer {
    pub async fn run(settings: &Settings) -> Result<Self, crate::error::Error> {
        use crate::schema::checkpoints::dsl::*;

        let client = ClientBuilder::default()
            .layer(RetryBackoffLayer::new(
                10,
                1000,
                settings
                    .fetcher_max_rps
                    .checked_mul(NonZeroU32::new(20).unwrap())
                    .unwrap()
                    .get() as u64,
            ))
            .ws(WsConnect::new(&settings.rpc_url))
            .await?;
        let provider = IndexerProvider::new(
            ProviderBuilder::new()
                .disable_recommended_fillers()
                .connect_client(client),
        )
        .await?;

        let chain_id_ = provider.get_chain_id().await?;

        let mut db_conn = SqliteConnection::establish(&settings.db_url)?;
        db_conn.batch_execute("PRAGMA journal_mode = WAL;")?;

        let start_block: u64 = checkpoints
            .select(Checkpoint::as_select())
            .filter(chain_id.eq(chain_id_ as i32))
            .first(&mut db_conn)?
            .last_saved_block_number
            .map(|n| n as u64)
            .unwrap_or(0);

        let block_fetcher = BlockFetcher::fetch(
            provider.clone(),
            BlockFetcherParams {
                max_concurrency: settings.fetcher_max_concurrency,
                max_rps: settings.fetcher_max_rps,
                start_block,
            },
        )
        .await?;

        let res = Self {
            provider,
            chain_id: chain_id_,
            db_conn,
            block_fetcher,
        };

        Ok(res)
    }

    async fn process_blocks(&self) {}
}
