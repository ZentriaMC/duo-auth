pub mod client;
pub mod errors;
pub mod request;
pub mod types;

pub(crate) type Error = Box<dyn std::error::Error + Send + Sync>;

pub use client::DuoClient;
