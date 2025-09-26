use openidconnect::DiscoveryError;
use openidconnect::reqwest;

use openidconnect::HttpClientError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Failed to parse environment variable:{0} \n {1:?}")]
    ParseEnvVar(String, url::ParseError),
    #[error("Failed to build HTTP client {0:?}")]
    Client(#[from] reqwest::Error),
    #[error("Failed to discover OpenID Provider {0:?}")]
    Provider(#[from] DiscoveryError<HttpClientError<openidconnect::reqwest::Error>>),
    #[error("Failed to parse URL {0:?}")]
    ParseUrl(#[from] url::ParseError),
}
