use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

const LOGIN_URL: &str = "https://login.questrade.com/oauth2/token";

#[derive(Debug)]
pub enum QuestradeAPIError {
    RequestError(reqwest::Error),
    JSONError(serde_json::Error),
}

impl Display for QuestradeAPIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            QuestradeAPIError::RequestError(err) => write!(f, "Request error: {}", err),
            QuestradeAPIError::JSONError(err) => write!(f, "JSON error: {}", err),
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

#[derive(Debug, Serialize, Deserialize)]
struct AccountsAPIResponse {
    accounts: Vec<Account>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    #[serde(rename = "type")]
    account_type: String,

    #[serde(rename = "number")]
    pub id: String,

    status: String,
    is_primary: bool,
    is_billing: bool,
    client_account_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuth2Token {
    access_token: String,
    token_type: String,
    expires_in: u16,
    pub refresh_token: String,
    api_server: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PositionsAPIResponse {
    positions: Vec<Position>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub symbol: String,
    symbol_id: u32,
    pub open_quantity: f64,
    closed_quantity: f64,
    current_market_value: f64,
    current_price: f64,
    average_entry_price: f64,
    closed_pnl: f64,
    open_pnl: f64,
    total_cost: f64,
    is_real_time: bool,
    is_under_reorg: bool,
}

pub fn get_oauth2_token(
    client: &reqwest::blocking::Client,
    refresh_token: &str,
) -> Result<OAuth2Token, QuestradeAPIError> {
    let mut params = HashMap::new();
    params.insert("grant_type", "refresh_token");
    params.insert("refresh_token", refresh_token);

    let body = client.get(LOGIN_URL).form(&params).send()?.text()?;

    Ok(serde_json::from_str::<OAuth2Token>(&body)?)
}

pub fn get_accounts(
    client: &reqwest::blocking::Client,
    token: &OAuth2Token,
) -> Result<Vec<Account>, QuestradeAPIError> {
    let body = client
        .get(format!("{}v1/accounts", token.api_server))
        .bearer_auth(token.access_token.clone())
        .send()?
        .text()?;

    dbg!(&body);
    let json = serde_json::from_str::<AccountsAPIResponse>(&body)?;

    Ok(json.accounts)
}

pub fn get_positions(
    client: &reqwest::blocking::Client,
    token: &OAuth2Token,
    account_id: &str,
) -> Result<Vec<Position>, QuestradeAPIError> {
    let url = format!("{}v1/accounts/{}/positions", token.api_server, account_id);
    let body = client
        .get(&url)
        .bearer_auth(token.access_token.clone())
        .send()?
        .text()?;

    let json = serde_json::from_str::<PositionsAPIResponse>(&body)?;

    Ok(json.positions)
}
