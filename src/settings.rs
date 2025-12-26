use std::num::NonZeroU32;

pub struct Settings {
    pub db_url: String,
    pub rpc_url: String,
    pub fetcher_max_concurrency: usize,
    pub fetcher_max_rps: NonZeroU32,
}
