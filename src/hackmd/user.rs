use serde::{Deserialize, Serialize};

use super::{team::Team, Client};

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    id: String,
    name: String,
    email: Option<String>,
    #[serde(rename = "userPath")]
    user_path: String,
    photo: String,
    teams: Vec<Team>,
}

pub struct UserApi<'a> {
    client: &'a Client,
}

impl<'a> UserApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn me(&self) -> Result<User, Box<dyn std::error::Error>> {
        let me = self
            .client
            .get("/v1/me")
            .send()
            .await?
            .json::<User>()
            .await?;
        Ok(me)
    }
}
