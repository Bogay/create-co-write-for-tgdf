use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Read {
    #[serde(rename = "owner")]
    Owner,
    #[serde(rename = "signed_in")]
    SignedIn,
    #[serde(rename = "guest")]
    Guest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Write {
    #[serde(rename = "owner")]
    Owner,
    #[serde(rename = "signed_in")]
    SignedIn,
    #[serde(rename = "guest")]
    Guest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Comment {
    #[serde(rename = "disabled")]
    Disabled,
    #[serde(rename = "forbidden")]
    Forbidden,
    #[serde(rename = "owners")]
    Owners,
    #[serde(rename = "signed_in_users")]
    SignedInUsers,
    #[serde(rename = "everyone")]
    Everyone,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_serialize() {
        assert_eq!(json!(Read::Owner), "owner");
        assert_eq!(json!(Write::SignedIn), "signed_in");
        assert_eq!(json!(Comment::Everyone), "everyone");
    }
}
