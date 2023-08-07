pub mod client;
pub mod errors;
pub mod request;
pub mod response;
pub mod types;

pub(crate) type StdError = Box<dyn std::error::Error + Send + Sync>;

pub use client::DuoClient;
