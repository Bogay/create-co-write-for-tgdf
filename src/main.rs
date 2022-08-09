mod hackmd;
mod tgdf;

use clap::Parser;
use futures::future::try_join_all;
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tera::Tera;
use tgdf::Agenda;

struct CoWriteCreator<T> {
    client: hackmd::Client,
    agendas: Vec<Agenda>,
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
            agendas: vec![],
        }
    }

    pub fn add_session(&mut self, session: T) {
        self.sessions.push(session);
    }

    pub fn add_agenda(&mut self, agenda: Agenda) {
        self.agendas.push(agenda);
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
        let mut notes = notes.into_iter();

        let agendas = self
            .agendas
            .iter()
            .map(|a| {
                let periods = a
                    .periods
                    .iter()
                    .map(|p| {
                        let mut sessions = vec![];
                        for s in &p.sessions {
                            let mut s = json!(s);
                            s.as_object_mut()
                                .unwrap()
                                .entry("note_id")
                                .or_insert(json!(notes.next().unwrap().id));
                            sessions.push(s);
                        }
                        let mut p = json!(p);
                        p.as_object_mut()
                            .unwrap()
                            .entry("sessions")
                            .and_modify(|s| *s = json!(sessions));
                        p
                    })
                    .collect::<Vec<_>>();
                let mut a = json!(a);
                a.as_object_mut()
                    .unwrap()
                    .entry("periods")
                    .and_modify(|p| *p = json!(periods));
                a
            })
            .collect::<Vec<_>>();

        let category_content = Tera::one_off(
            &self.category_template,
            &tera::Context::from_value(json!({ "agendas": &agendas }))?,
            false,
        )?;
        self.client.note().create(&category_content).await?;

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
    let agendas = tgdf::fetch().await?;
    let mut creator = CoWriteCreator::<tgdf::Session>::new(
        &fs::read_to_string(&cli.token_path)?,
        fs::read_to_string("templates/category.tera")?,
        fs::read_to_string("templates/note.tera")?,
    )
    .await;

    for session in agendas.iter().map(|a| a.sessions()).flatten() {
        // FIXME: clone is not necessary
        creator.add_session(session.clone());
    }
    for agenda in agendas {
        creator.add_agenda(agenda);
    }

    creator.create().await
}
