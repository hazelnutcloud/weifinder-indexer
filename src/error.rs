use alloy::transports::{RpcError, TransportErrorKind};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("RPC error: {0}")]
    RpcError(#[from] RpcError<TransportErrorKind>),
    #[error("Database connection error")]
    DbConnectionError(#[from] diesel::ConnectionError),
    #[error("Database query error")]
    DbQueryError(#[from] diesel::result::Error),
}
