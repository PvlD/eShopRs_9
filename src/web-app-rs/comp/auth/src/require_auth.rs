use axum::{
    extract::{FromRequestParts, Path},
    http::{self, Extensions, Response, request::Parts},
};
//use async_trait::async_trait;

// An extractor that performs authorization.
pub struct RequireAuth2;
//#[async_trait]
impl<S> FromRequestParts<S> for RequireAuth2
where
    S: Send + Sync,
{
    type Rejection = Response<axum::body::Body>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        use leptos::server_fn::response::Res;

        let extensions = Extensions::from_request_parts(parts, state).await.map_err(|_e| {
            let mut res = http::Response::<axum::body::Body>::new(" Extensions::from_request_parts failed".into());
            *res.status_mut() = http::StatusCode::INTERNAL_SERVER_ERROR;
            res
        })?;

        let is_authenticated = crate::server::is_authenticated_from_extensions(&extensions);

        match is_authenticated {
            Ok(true) => Ok(Self),

            Ok(false) => {
                let path = path_from_request_parts(parts, state).await?;

                let data = serde_json::to_string(&app_err::AppError::Unauthorized).unwrap();
                let res = http::Response::<axum::body::Body>::error_response(&path, data.into());
                Err(res)
            }
            Err(e) => {
                let path = path_from_request_parts(parts, state).await?;
                let data = serde_json::to_string(&app_err::AppError::ServerFnError(e)).unwrap();
                let res = http::Response::<axum::body::Body>::error_response(&path, data.into());
                Err(res)
            }
        }
    }
}

async fn path_from_request_parts<S>(parts: &mut Parts, state: &S) -> Result<String, Response<axum::body::Body>>
where
    S: Send + Sync,
{
    let path: Path<String> = Path::from_request_parts(parts, state).await.map_err(|_e| {
        let mut res = http::Response::<axum::body::Body>::new("path_from_request_parts".into());
        *res.status_mut() = http::StatusCode::INTERNAL_SERVER_ERROR;
        res
    })?;
    Ok(path.0)
}
