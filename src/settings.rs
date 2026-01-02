use std::num::{NonZeroU32, NonZeroUsize};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub rpc_ws: String,
    pub fetcher_max_concurrency: NonZeroUsize,
    pub fetcher_max_rps: NonZeroU32,
    pub batch_save_size: NonZeroUsize,
}
