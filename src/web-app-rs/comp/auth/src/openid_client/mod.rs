mod error;
mod openid_client;

pub use error::Error;
pub use openid_client::{OCClient, create_from_env};
