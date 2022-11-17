use super::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Team {
    id: String,
    #[serde(rename = "ownerId")]
    owner_id: String,
    path: String,
    name: String,
    logo: String,
    description: String,
    visibility: String,
    #[serde(rename = "createdAt")]
    created_at: u64,
}

pub struct TeamApi<'a> {
    client: &'a Client,
}

impl<'a> TeamApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn get_list(&self) -> Result<Vec<Team>, Box<dyn std::error::Error>> {
        let teams = self
            .client
            .get("/teams")
            .send()
            .await?
            .json::<Vec<Team>>()
            .await?;

        Ok(teams)
    }
}
