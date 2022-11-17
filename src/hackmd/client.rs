use super::note::NoteApi;
use reqwest::{RequestBuilder, Url};

#[derive(Debug, Clone)]
pub struct Client {
    client: reqwest::Client,
    base_url: Url,
    token: String,
}

macro_rules! impl_http_method {
    ($method: ident) => {
        #[allow(dead_code)]
        pub(crate) fn $method(&self, path: &str) -> RequestBuilder {
            let mut url = self.base_url.clone();
            url.set_path(path);
            self.client.$method(url).bearer_auth(&self.token)
        }
    };
}

impl Client {
    pub async fn new(token: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let client = Self {
            client,
            base_url: "https://api.hackmd.io".parse::<Url>()?,
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
        self.get("/v1/me").send().await?;

        Ok(())
    }

    pub fn note(&self) -> NoteApi {
        NoteApi::new(self)
    }
}
