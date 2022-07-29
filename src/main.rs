mod hackmd;
mod tgdf;

use futures::future::try_join_all;
use iter_tools::Itertools;
use serde::Serialize;
use serde_json::json;
use std::fs;
use tera::Tera;

struct CoWriteCreator<T> {
    client: hackmd::Client,
    sessions: Vec<T>,
    category_template: String,
    note_template: String,
}

impl<T> CoWriteCreator<T>
where
    T: Serialize,
{
    pub async fn new(token: &str, category_template: String, note_template: String) -> Self {
        let client = hackmd::Client::new(token).await.unwrap();

        Self {
            client,
            category_template,
            note_template,
            sessions: vec![],
        }
    }

    pub fn add_session(&mut self, session: T) {
        self.sessions.push(session);
    }

    pub async fn create(&self) -> Result<(), Box<dyn std::error::Error>> {
        let note_contents = self
            .sessions
            .iter()
            .map(|session| self.gen_session_note_content(session))
            .collect::<Vec<_>>();
        let notes = try_join_all(
            note_contents
                .iter()
                .map(|note_content| self.client.create_note(note_content)),
        )
        .await?;

        let ctx: Vec<_> = self
            .sessions
            .iter()
            .zip(&notes)
            .map(|(session, note)| json!({"session": session, "note": note}))
            .collect();

        Tera::one_off(
            &self.category_template,
            &tera::Context::from_value(json!({ "ctx": &ctx })).unwrap(),
            false,
        )
        .unwrap();

        let ids = notes.iter().map(|note| &note.id).join("\n");
        println!("{}", ids);

        Ok(())
    }

    pub(crate) fn gen_session_note_content(&self, session: &T) -> String {
        Tera::one_off(
            &self.note_template,
            &tera::Context::from_serialize(session).unwrap(),
            false,
        )
        .unwrap()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sessions = tgdf::fetch().await?;
    let mut creator = CoWriteCreator::<tgdf::Session>::new(
        &fs::read_to_string("token.txt").unwrap(),
        fs::read_to_string("templates/category.tera").unwrap(),
        fs::read_to_string("templates/note.tera").unwrap(),
    )
    .await;

    for session in sessions {
        creator.add_session(session);
    }

    creator.create().await
}
