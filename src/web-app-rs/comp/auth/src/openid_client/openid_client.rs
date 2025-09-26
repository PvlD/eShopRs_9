use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::{ClientId, ClientSecret, IssuerUrl, RedirectUrl};
use openidconnect::{EndpointMaybeSet, EndpointNotSet, EndpointSet, reqwest};

use anyhow::Result;

use url::Url;

pub type OCClient = CoreClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointMaybeSet, EndpointMaybeSet>;

const KEY_OPENID_CLIENT_ID: &str = "OPENID_CLIENT_ID";
const KEY_OPENID_CLIENT_SECRET: &str = "OPENID_CLIENT_SECRET";
const KEY_OPENID_ISSUER_URL: &str = "IdentityUrl";
const KEY_OPENID_REDIRECT_URL: &str = "OPENID_REDIRECT_URL";

fn env_var(name: &str) -> Result<String> {
    Ok(std::env::var(name).map_err(|_| super::error::Error::MissingEnvVar(name.to_string()))?)
}

pub async fn create_from_env(http_client: reqwest::Client, site_url: Url) -> Result<OCClient> {
    let client_id = ClientId::new(env_var(KEY_OPENID_CLIENT_ID)?);
    let client_secret = ClientSecret::new(env_var(KEY_OPENID_CLIENT_SECRET)?);

    let redirect_url = env_var(KEY_OPENID_REDIRECT_URL)?;

    let redirect_url = site_url.join(redirect_url.as_str()).map_err(|err: url::ParseError| super::error::Error::ParseUrl(err))?;

    let redirect_url = RedirectUrl::from_url(redirect_url);

    let issuer_url = IssuerUrl::new(env_var(KEY_OPENID_ISSUER_URL)?).map_err(|err: url::ParseError| super::error::Error::ParseEnvVar(KEY_OPENID_ISSUER_URL.to_string(), err))?;

    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, &http_client).await.map_err(super::error::Error::Provider)?;

    let client = CoreClient::from_provider_metadata(provider_metadata, client_id, Some(client_secret)).set_redirect_uri(redirect_url);

    Ok(client)
}
