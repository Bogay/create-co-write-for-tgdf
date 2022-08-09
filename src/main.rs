mod hackmd;
mod tgdf;

use clap::Parser;
use futures::future::try_join_all;
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
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
            .collect::<Result<Vec<_>, _>>()?;
        let note_api = self.client.note();
        let notes = try_join_all(
            note_contents
                .iter()
                .map(|note_content| note_api.create(note_content)),
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
            &tera::Context::from_value(json!({ "ctx": &ctx }))?,
            false,
        )?;

        Ok(())
    }

    pub(crate) fn gen_session_note_content(&self, session: &T) -> tera::Result<String> {
        Tera::one_off(
            &self.note_template,
            &tera::Context::from_serialize(session)?,
            false,
        )
    }
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Path to the HackMD API token
    #[clap(long, value_parser, value_name = "FILE")]
    token_path: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let sessions = tgdf::fetch().await?;
    let mut creator = CoWriteCreator::<tgdf::Session>::new(
        &fs::read_to_string(&cli.token_path)?,
        fs::read_to_string("templates/category.tera")?,
        fs::read_to_string("templates/note.tera")?,
    )
    .await;

    for session in sessions {
        creator.add_session(session);
    }

    creator.create().await
}
