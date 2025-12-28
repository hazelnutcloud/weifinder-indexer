use std::num::{NonZeroU32, NonZeroUsize};

#[derive(Debug)]
pub struct Settings {
    pub rpc_url: String,
    pub fetcher_max_concurrency: NonZeroUsize,
    pub fetcher_max_rps: NonZeroU32,
    pub batch_save_size: NonZeroUsize,
}
