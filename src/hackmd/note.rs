use super::{permission, Client};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize)]
pub struct NoteBuilder<'a> {
    #[serde(skip)]
    client: &'a Client,
    title: Option<String>,
    content: Option<String>,
    comment_permission: Option<permission::Comment>,
    read_permission: Option<permission::Read>,
    write_permission: Option<permission::Write>,
}

impl<'a> NoteBuilder<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self {
            client,
            title: None,
            content: None,
            comment_permission: None,
            read_permission: None,
            write_permission: None,
        }
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn content(mut self, content: String) -> Self {
        self.content = Some(content);
        self
    }

    pub fn comment_permission(mut self, comment_permission: permission::Comment) -> Self {
        self.comment_permission = Some(comment_permission);
        self
    }

    pub fn read_permission(mut self, read_permission: permission::Read) -> Self {
        self.read_permission = Some(read_permission);
        self
    }

    pub fn write_permission(mut self, write_permission: permission::Write) -> Self {
        self.write_permission = Some(write_permission);
        self
    }

    pub async fn done(self) -> Result<Note, Box<dyn std::error::Error>> {
        let payload = json!(self);
        let response = self.client.post("/v1/notes").json(&payload).send().await?;
        let note = response.json::<Note>().await?;

        Ok(note)
    }
}

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

    pub async fn get(&self, id: &str) -> Result<Note, Box<dyn std::error::Error>> {
        let note = self
            .client
            .get(&format!("/v1/notes/{}", id))
            .send()
            .await?
            .json::<Note>()
            .await?;

        Ok(note)
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

    pub fn builder(&self) -> NoteBuilder {
        NoteBuilder::new(&self.client)
    }

    pub async fn delete(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.client
            .delete(&format!("/v1/notes/{}", id))
            .send()
            .await?;

        Ok(())
    }
}
