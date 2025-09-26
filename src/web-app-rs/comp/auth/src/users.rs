use axum_login::{AuthUser, AuthnBackend, UserId};
use openidconnect::{
    AdditionalClaims, AuthenticationFlow, AuthorizationCode, CsrfToken, Nonce, OAuth2TokenResponse, Scope, UserInfoClaims,
    core::{CoreGenderClaim, CoreIdTokenClaims, CoreIdTokenVerifier, CoreResponseType},
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use url::Url;

use crate::openid_client::OCClient;

#[derive(Debug, Deserialize, Serialize)]
struct AppClaims {
    // Deprecated and thus optional as it might be removed in the futre
    address_street: Option<String>,
    address_city: Option<String>,
    address_state: Option<String>,
    address_zip_code: Option<String>,
    address_country: Option<String>,
}
impl AdditionalClaims for AppClaims {}

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    id: i64,
    pub sub: String,
    pub username: String,
    pub email: String,

    pub access_token: String,
    pub refresh_token: Option<String>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub zip: Option<String>,
}

// Here we've implemented `Debug` manually to avoid accidentally logging the
// access token.
impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("email", &self.email)
            .field("access_token", &"[redacted]")
            .field("refresh_token", &"[redacted]")
            .field("street", &self.street)
            .field("city", &self.city)
            .field("state", &self.state)
            .field("country", &self.country)
            .field("zip", &self.zip)
            .finish()
    }
}

impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.access_token.as_bytes()
    }
}

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub code: String,
    pub old_state: CsrfToken,
    pub new_state: CsrfToken,
    pub nonce: Nonce,
}

#[cfg(feature = "hydrate")]
#[derive(Debug, Deserialize)]
struct UserInfo {
    #[serde(rename(deserialize = "name"))]
    login: String,
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error(transparent)]
    Sqlx(sqlx::Error),

    #[error(transparent)]
    Reqwest(reqwest::Error),

    #[error(transparent)]
    RequestTokenError(openidconnect::RequestTokenError<openidconnect::HttpClientError<reqwest::Error>, openidconnect::StandardErrorResponse<openidconnect::core::CoreErrorResponseType>>),

    #[error(transparent)]
    ConfigurationError(openidconnect::ConfigurationError),

    #[error("UserInfoError: {0}")]
    UserInfoError(String),

    #[error("Server did not return an ID token")]
    ReturnIDToken(),

    #[error(transparent)]
    ClaimsVerificationError(openidconnect::ClaimsVerificationError),

    #[error("Calaim {0} not found in token")]
    CalaimNotFound(String),
}

#[derive(Debug, Clone)]
pub struct Backend {
    db: SqlitePool,
    client: OCClient,
    http_client: reqwest::Client,
}

impl Backend {
    pub fn new(db: SqlitePool, client: OCClient, http_client: reqwest::Client) -> Self {
        Self { db, client, http_client }
    }

    pub fn authorize_url_with_scopes(&self, vec: Vec<&str>) -> (Url, CsrfToken, Nonce) {
        let mut au_rqst = self.client.authorize_url(AuthenticationFlow::<CoreResponseType>::AuthorizationCode, CsrfToken::new_random, Nonce::new_random);

        let scopes: Vec<Scope> = vec.iter().map(|s| Scope::new(s.to_string())).collect();

        au_rqst = au_rqst.add_scopes(scopes);

        au_rqst.url()
    }
}

impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = BackendError;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        // Ensure the CSRF state has not been tampered with.
        if creds.old_state.secret() != creds.new_state.secret() {
            return Ok(None);
        };

        let http_client = self.http_client.clone();

        // Process authorization code, expecting a token response back.
        let token_response = self
            .client
            .exchange_code(AuthorizationCode::new(creds.code))
            .map_err(|err: openidconnect::ConfigurationError| Self::Error::ConfigurationError(err))?
            .request_async(&http_client)
            .await
            .map_err(Self::Error::RequestTokenError)?;

        let id_token_verifier: CoreIdTokenVerifier = self.client.id_token_verifier();
        let id_token_claims: &CoreIdTokenClaims = token_response
            .extra_fields()
            .id_token()
            .ok_or_else(|| Self::Error::ReturnIDToken())?
            .claims(&id_token_verifier, &creds.nonce)
            .map_err(Self::Error::ClaimsVerificationError)?;

        println!(" returned ID token: {:?}", id_token_claims);

        let sub = id_token_claims.subject().to_string();

        let name = id_token_claims.name().and_then(|n| n.get(None).map(|f| f.to_string())).ok_or(Self::Error::CalaimNotFound("name".to_string()))?;

        let email = id_token_claims.email().map(|n| n.to_string()).ok_or(Self::Error::CalaimNotFound("email".to_string()))?;

        let userinfo_claims: UserInfoClaims<AppClaims, CoreGenderClaim> = self
            .client
            .user_info(token_response.access_token().to_owned(), None)
            .map_err(|err| Self::Error::UserInfoError(err.to_string()))?
            .request_async(&http_client)
            .await
            .map_err(|err| Self::Error::UserInfoError(err.to_string()))?;

        let AppClaims {
            address_street,
            address_city,
            address_state,
            address_zip_code,
            address_country,
        } = userinfo_claims.additional_claims();

        //println!(" returned user info: {:?}, {:?}, {:?}, {:?}, {:?}", address_street, address_city, address_state, address_zip_code, address_country);

        // Persist user in our database so we can use `get_user`.
        let user = sqlx::query_as(
            r#"
            insert into users (sub, username, email, access_token, refresh_token, street, city, state, country, zip)
            values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            on conflict(username) do update
            set
            sub = excluded.sub,
            email = excluded.email,
            access_token = excluded.access_token,
            refresh_token = excluded.refresh_token,
            street = excluded.street,
            city = excluded.city ,
            state =excluded.state,
            country = excluded.country ,
            zip = excluded.zip
            returning *
            "#,
        )
        .bind(sub)
        .bind(name)
        .bind(email)
        .bind(token_response.access_token().secret())
        .bind(token_response.refresh_token().map(|rt| rt.secret()))
        .bind(address_street)
        .bind(address_city)
        .bind(address_state)
        .bind(address_zip_code)
        .bind(address_country)
        .fetch_one(&self.db)
        .await
        .map_err(Self::Error::Sqlx)?;

        //println!(" returned user: {:#?}", user);
        Ok(Some(user))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        sqlx::query_as("select * from users where id = ?").bind(user_id).fetch_optional(&self.db).await.map_err(Self::Error::Sqlx)
    }
}

// We use a type alias for convenience.
//
// Note that we've supplied our concrete backend here.
pub type AuthSession = axum_login::AuthSession<Backend>;
