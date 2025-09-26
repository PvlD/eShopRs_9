use std::sync::Arc;

use async_trait::async_trait;

use crate::user::UserInfo;

#[derive(Debug)]
pub struct UserAddressInfo {
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub zip: Option<String>,
}

#[async_trait]
pub trait AuthService: Send + Sync {
    async fn is_logged_in(&self) -> Result<Option<UserInfo>, crate::AppError>;
    async fn get_user_address_info(&self) -> Result<UserAddressInfo, crate::AppError>;
    async fn get_buyer_id(&self) -> Result<String, crate::AppError>;
    async fn get_user_name(&self) -> Result<String, crate::AppError>;
}
#[derive(Clone)]
pub struct AuthServiceContext {
    pub service: Arc<dyn AuthService>,
}
