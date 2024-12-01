use crate::db::{DatabaseAPI, RefreshToken};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

const LOGIN_URL: &str = "https://login.questrade.com/oauth2/token";

#[derive(Debug)]
pub enum QuestradeAPIError {
    RequestError(reqwest::Error),
    JSONError(serde_json::Error),
    APIError(String),
    DBError(sqlx::Error),
}

impl Display for QuestradeAPIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            QuestradeAPIError::RequestError(err) => write!(f, "Request error: {}", err),
            QuestradeAPIError::JSONError(err) => write!(f, "JSON error: {}", err),
            QuestradeAPIError::APIError(msg) => write!(f, "Questrade API error: {}", msg),
            QuestradeAPIError::DBError(err) => write!(f, "DB error: {}", err),
        }
    }
}

impl From<reqwest::Error> for QuestradeAPIError {
    fn from(err: reqwest::Error) -> Self {
        QuestradeAPIError::RequestError(err)
    }
}

impl From<serde_json::Error> for QuestradeAPIError {
    fn from(err: serde_json::Error) -> Self {
        QuestradeAPIError::JSONError(err)
    }
}

impl From<sqlx::Error> for QuestradeAPIError {
    fn from(err: sqlx::Error) -> Self {
        QuestradeAPIError::DBError(err)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuth2Token {
    access_token: String,
    token_type: String,
    expires_in: u16,
    pub refresh_token: String,
    api_server: String,
}

pub struct QuestradeAPI {
    client: reqwest::Client,
    token: OAuth2Token,
}

impl QuestradeAPI {
    pub async fn new(db: DatabaseAPI) -> Result<Self, QuestradeAPIError> {
        let client = reqwest::Client::new();
        let old_refresh_token = match db.get_refresh_token().await {
            Ok(token) => token,
            Err(err) => {
                eprintln!("Refresh token not found in database. Run --help to see how to add one.");
                return Err(QuestradeAPIError::DBError(err));
            }
        };

        let token = Self::get_oauth2_token(&client, &old_refresh_token).await?;
        db.update_refresh_token(&old_refresh_token, &token.refresh_token)
            .await?;

        Ok(Self { client, token })
    }

    async fn get_oauth2_token(
        client: &reqwest::Client,
        refresh_token: &RefreshToken,
    ) -> Result<OAuth2Token, QuestradeAPIError> {
        let mut params = HashMap::new();
        params.insert("grant_type", "refresh_token");
        params.insert("refresh_token", &refresh_token.refresh_token);

        let body = client
            .get(LOGIN_URL)
            .form(&params)
            .send()
            .await?
            .text()
            .await?;

        Ok(serde_json::from_str::<OAuth2Token>(&body)?)
    }

    pub async fn make_request(&self, path: String) -> Result<String, QuestradeAPIError> {
        let resp = self
            .client
            .get(format!("{}{}", self.token.api_server, path))
            .bearer_auth(&self.token.access_token)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(QuestradeAPIError::APIError(resp.text().await?));
        }

        Ok(resp.text().await?)
    }
}
