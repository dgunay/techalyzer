//! Generic container for Secrets.

// TODO: use the secrecy crate or something similar to protect these in memory.

/// Represents something like an API key used to access a market data source.
pub struct Secret {
    pub data: Option<String>,
}
