use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

const LOGIN_URL: &str = "https://login.questrade.com/oauth2/token";

#[derive(Debug)]
pub enum QuestradeAPIError {
    RequestError(reqwest::Error),
    JSONError(serde_json::Error),
    APIError(Option<String>),
}

impl Display for QuestradeAPIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            QuestradeAPIError::RequestError(err) => write!(f, "Request error: {}", err),
            QuestradeAPIError::JSONError(err) => write!(f, "JSON error: {}", err),
            QuestradeAPIError::APIError(msg) => write!(f, "Questrade API error: {}", msg.as_deref().unwrap_or("")),
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
    pub type_: String,

    #[serde(rename = "number")]
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BalancesAPIResponse {
    per_currency_balances: Vec<Balance>,
    combined_balances: Vec<Balance>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    pub currency: String,
    pub cash: f64,
    pub market_value: f64,
    pub total_equity: f64,
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
    pub symbol_id: u32,
    pub open_quantity: f64,
    pub closed_quantity: f64,
    pub current_market_value: f64,
    pub current_price: f64,
    pub average_entry_price: f64,
    pub closed_pnl: f64,
    pub open_pnl: f64,
    pub total_cost: f64
}

#[derive(Debug, Serialize, Deserialize)]
struct SymbolsAPIResponse {
    symbols: Vec<Symbol>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Symbol {
    pub symbol: String,
    pub symbol_id: u32,
    pub dividend: f64,
    pub yield_: f64,
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
    let resp = client
        .get(format!("{}v1/accounts", token.api_server))
        .bearer_auth(token.access_token.clone())
        .send()?;

    if !resp.status().is_success() {
        return Err(QuestradeAPIError::APIError(Some(resp.text()?)));
    }

    let body = resp.text()?;
    let json = serde_json::from_str::<AccountsAPIResponse>(&body)?;

    Ok(json.accounts)
}

pub fn get_balances(
    client: &reqwest::blocking::Client,
    token: &OAuth2Token,
    account_id: &str,
) -> Result<Vec<Balance>, QuestradeAPIError> {
    let url = format!("{}v1/accounts/{}/balances", token.api_server, account_id);
    let resp = client
        .get(&url)
        .bearer_auth(token.access_token.clone())
        .send()?;

    if !resp.status().is_success() {
        return Err(QuestradeAPIError::APIError(Some(resp.text()?)));
    }

    let body = resp.text()?;
    let json = serde_json::from_str::<BalancesAPIResponse>(&body)?;

    Ok(json.combined_balances)
}

pub fn display_balances(balances: &Vec<Balance>) {
    println!("{:<10} | {:<10} | {:<15} | {:>15}", "Currency", "Cash", "Market Value", "Total Equity");
    println!("{}", "-".repeat(59));
    for balance in balances {
        println!("{:<10} | {:<10.2} | {:<15.2} | {:>15.2}", balance.currency, balance.cash, balance.market_value, balance.total_equity);
    }
    println!();
}

fn get_positions(
    client: &reqwest::blocking::Client,
    token: &OAuth2Token,
    account_id: &str,
) -> Result<Vec<Position>, QuestradeAPIError> {
    let url = format!("{}v1/accounts/{}/positions", token.api_server, account_id);
    let resp = client
        .get(&url)
        .bearer_auth(token.access_token.clone())
        .send()?;

    if !resp.status().is_success() {
        return Err(QuestradeAPIError::APIError(Some(resp.text()?)));
    }

    let body = resp.text()?;
    let json = serde_json::from_str::<PositionsAPIResponse>(&body)?;

    Ok(json.positions)
}

fn get_symbol(
    client: &reqwest::blocking::Client,
    token: &OAuth2Token,
    symbol_id: u32,
) -> Result<Symbol, QuestradeAPIError> {
    let url = format!("{}v1/symbols/{}", token.api_server, symbol_id);
    let resp = client
        .get(&url)
        .bearer_auth(token.access_token.clone())
        .send()?;

    if !resp.status().is_success() {
        return Err(QuestradeAPIError::APIError(Some(resp.text()?)));
    }

    let body = resp.text()?;
    let json = serde_json::from_str::<SymbolsAPIResponse>(&body)?;

    if let Some(symbol) = json.symbols.first() {
        return Ok(symbol.clone());
    }

    Err(QuestradeAPIError::APIError(None))
}

pub fn get_positions_and_symbols(
    client: &reqwest::blocking::Client,
    token: &OAuth2Token,
    account_id: &str,
) -> Result<(Vec<Position>, HashMap<u32, Symbol>), QuestradeAPIError> {
    let positions = get_positions(client, token, account_id)?;
    let mut symbols = HashMap::new();

    for position in positions.iter() {
        let symbol = get_symbol(client, token, position.symbol_id)?;
        symbols.entry(symbol.symbol_id).or_insert(symbol);
    }

    Ok((positions, symbols))
}

pub fn display_positions_with_dividends(positions: &Vec<Position>, symbols: &HashMap<u32, Symbol>) {
    println!("---Positions---");
    println!("{:<10} | {:<10} | {:<10} | {:<15} | {:<15} | {:<15} | {:<10} | {:<10} | {:>10}", "Symbol", "Quantity", "Avg Price", "Book Cost", "Market Price", "Market Value", "Dividend", "Yield", "P&L");
    println!("{}", "-".repeat(129));

    let mut total_cost = 0.0;
    let mut total_mkt_val = 0.0;
    let mut total_pnl = 0.0;

    for position in positions {
        let (dividend, yield_) = if let Some(symbol) = symbols.get(&position.symbol_id) {
            (symbol.dividend, symbol.yield_)
        } else {
            (0.0, 0.0)
        };

        let quantity = if position.closed_quantity == 0.0 {
            position.open_quantity
        } else {
            position.closed_quantity
        };

        let pnl = if position.closed_pnl == 0.0 {
            position.open_pnl
        } else {
            position.closed_pnl
        };

        total_cost += position.total_cost;
        total_mkt_val += position.current_market_value;
        total_pnl += pnl;

        println!(
            "{:<10} | {:<10} | {:<10.2} | {:<15.2} | {:<15.2} | {:<15.2} | {:<10.4} | {:<10.2} | {:>10.2}",
            position.symbol, quantity, position.average_entry_price, position.total_cost, position.current_price, position.current_market_value, dividend, yield_, pnl
        );
    }

    println!("{}", "=".repeat(129));
    println!("{:<10} | {:<10} | {:<10} | {:<15.2} | {:<15} | {:<15.2} | {:<10} | {:<10} | {:>10.2}", "Total", "", "", total_cost, "", total_mkt_val, "", "", total_pnl);
    println!();
}
