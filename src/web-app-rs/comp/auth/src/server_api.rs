use leptos::server;

use crate::user::UserInfo;

#[server(prefix = "/user")]
pub async fn is_logged_in() -> Result<Option<UserInfo>, crate::AppError> {
    use crate::service::AuthServiceContext;
    use leptos::prelude::expect_context;

    let auth_service_context = expect_context::<AuthServiceContext>();
    auth_service_context.service.is_logged_in().await
}
