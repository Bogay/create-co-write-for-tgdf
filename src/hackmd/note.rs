use super::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub tags: Vec<String>,
}

pub struct NoteAPI<'a> {
    client: &'a Client,
}

impl<'a> NoteAPI<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn get_list(&self) -> Result<Vec<Note>, Box<dyn std::error::Error>> {
        let notes = self
            .client
            .get("/v1/notes")
            .send()
            .await?
            .json::<Vec<Note>>()
            .await?;

        Ok(notes)
    }

    pub async fn create(&self, content: &str) -> Result<Note, Box<dyn std::error::Error>> {
        let payload = json!({
            "content": content,
            "readPermission": "guest",
            "writePermission": "signed_in",
        });
        let response = self.client.post("/v1/notes").json(&payload).send().await?;
        let note = response.json::<Note>().await?;

        Ok(note)
    }
}
