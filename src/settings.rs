use std::fmt;
use std::num::{NonZeroU32, NonZeroUsize};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub rpc_ws: String,
    pub fetcher_max_blocks_per_second: NonZeroU32,
    pub batch_save_size: NonZeroUsize,
    pub catalog_db_url: String,
    pub s3_endpoint: String,
    pub s3_access_key_id: String,
    pub s3_secret_access_key: String,
    pub s3_bucket: String,
    pub checkpoint_db_path: String,
}

/// `Debug` is implemented by hand instead of derived because the settings are logged at
/// startup: the S3 credentials and the catalog URL (which carries a database password)
/// must never reach the logs.
impl fmt::Debug for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const REDACTED: &str = "<redacted>";

        f.debug_struct("Settings")
            .field("rpc_ws", &self.rpc_ws)
            .field(
                "fetcher_max_blocks_per_second",
                &self.fetcher_max_blocks_per_second,
            )
            .field("batch_save_size", &self.batch_save_size)
            .field("catalog_db_url", &REDACTED)
            .field("s3_endpoint", &self.s3_endpoint)
            .field("s3_access_key_id", &REDACTED)
            .field("s3_secret_access_key", &REDACTED)
            .field("s3_bucket", &self.s3_bucket)
            .field("checkpoint_db_path", &self.checkpoint_db_path)
            .finish()
    }
}
