use super::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UserInfo {
    pub name: String,
    pub email: String,
}
