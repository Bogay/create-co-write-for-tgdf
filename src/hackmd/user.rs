use super::Client;

pub struct UserApi<'a> {
    client: &'a Client,
}

impl<'a> UserApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn me(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.client.get("/v1/me").send().await?;
        Ok(())
    }
}
