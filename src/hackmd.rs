use reqwest::{IntoUrl, RequestBuilder};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub enum ReadPermission {
    Owner,
    SignedIn,
    Guest,
}
pub enum WritePermission {
    Owner,
    SignedIn,
    Guest,
}

pub enum CommentPermission {
    Disabled,
    Forbidden,
    Owners,
    SignedInUsers,
    Everyone,
}

pub struct Client {
    client: reqwest::Client,
    base_url: String,
    token: String,
}

macro_rules! impl_http_method {
    ($method: ident) => {
        #[allow(dead_code)]
        fn $method<T: IntoUrl>(&self, url: T) -> RequestBuilder {
            self.client.$method(url).bearer_auth(&self.token)
        }
    };
}

impl Client {
    pub async fn new(token: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let client = Self {
            client,
            base_url: "https://api.hackmd.io".to_string(),
            token: token.to_string(),
        };
        client.get_me().await?;
        Ok(client)
    }

    impl_http_method!(get);
    impl_http_method!(post);
    impl_http_method!(put);
    impl_http_method!(patch);
    impl_http_method!(delete);

    async fn get_me(&self) -> Result<(), Box<dyn std::error::Error>> {
        let _ = self.get(format!("{}/v1/me", self.base_url)).send().await?;

        Ok(())
    }

    pub async fn create_note(&self, content: &str) -> Result<Note, Box<dyn std::error::Error>> {
        let payload = json!({
            "content": content,
            "readPermission": "guest",
            "writePermission": "signed_in",
        });
        let response = self
            .post(format!("{}/v1/notes", self.base_url))
            .json(&payload)
            .send()
            .await?;
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