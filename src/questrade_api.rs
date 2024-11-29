use colored::{ColoredString, Colorize};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env, fmt::Display};

use crate::db::{get_db_collection, get_refresh_token, update_refresh_token};

const LOGIN_URL: &str = "https://login.questrade.com/oauth2/token";

#[derive(Debug)]
pub enum QuestradeAPIError {
    RequestError(reqwest::Error),
    JSONError(serde_json::Error),
    APIError(String),
    DBError(mongodb::error::Error),
}

impl Display for QuestradeAPIError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            QuestradeAPIError::RequestError(err) => write!(f, "Request error: {}", err),
            QuestradeAPIError::JSONError(err) => write!(f, "JSON error: {}", err),
            QuestradeAPIError::APIError(msg) => write!(f, "Questrade API error: {}", msg),
            QuestradeAPIError::DBError(err) => write!(f, "MongoDB error: {}", err),
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

impl From<mongodb::error::Error> for QuestradeAPIError {
    fn from(err: mongodb::error::Error) -> Self {
        QuestradeAPIError::DBError(err)
    }
}

pub struct QuestradeAPI {
    client: reqwest::blocking::Client,
    token: OAuth2Token,
}

impl QuestradeAPI {
    pub fn new(client: reqwest::blocking::Client) -> Result<Self, QuestradeAPIError> {
        let db_password = env::vars()
            .find(|(key, _)| key == "DB_PASSWORD")
            .expect("DB_PASSWORD must be supplied in .env")
            .1;
        let coll = get_db_collection(&db_password)?;
        let refresh_token = get_refresh_token(&coll)?.expect("No refresh token found in database");

        let token = Self::get_oauth2_token(&client, &refresh_token)?;
        update_refresh_token(&coll, &refresh_token, &token.refresh_token)?;

        Ok(Self { client, token })
    }

    pub fn get_accounts(&self) -> Result<Vec<Account>, QuestradeAPIError> {
        let url = format!("{}v1/accounts", self.token.api_server);
        let resp = self.make_request(&url)?;
        let accounts = serde_json::from_str::<Accounts>(&resp)?;

        Ok(accounts.accounts)
    }

    pub fn get_balances(&self, account_id: &str) -> Result<Balances, QuestradeAPIError> {
        let url = format!(
            "{}v1/accounts/{}/balances",
            self.token.api_server, account_id
        );
        let resp = self.make_request(&url)?;
        let balances = serde_json::from_str::<Balances>(&resp)?;

        Ok(balances)
    }

    fn get_oauth2_token(
        client: &reqwest::blocking::Client,
        refresh_token: &str,
    ) -> Result<OAuth2Token, QuestradeAPIError> {
        let mut params = HashMap::new();
        params.insert("grant_type", "refresh_token");
        params.insert("refresh_token", refresh_token);

        let body = client.get(LOGIN_URL).form(&params).send()?.text()?;

        Ok(serde_json::from_str::<OAuth2Token>(&body)?)
    }

    fn get_positions(&self, account_id: &str) -> Result<Vec<Position>, QuestradeAPIError> {
        let url = format!(
            "{}v1/accounts/{}/positions",
            self.token.api_server, account_id
        );
        let resp = self.make_request(&url)?;
        let positions = serde_json::from_str::<Positions>(&resp)?;

        Ok(positions.positions)
    }

    fn get_symbol(&self, symbol_id: u32) -> Result<Symbol, QuestradeAPIError> {
        let url = format!("{}v1/symbols/{}", self.token.api_server, symbol_id);
        let resp = self.make_request(&url)?;
        let symbols = serde_json::from_str::<Symbols>(&resp)?;

        if let Some(symbol) = symbols.symbols.first() {
            return Ok(symbol.clone());
        }

        Err(QuestradeAPIError::APIError("Symbol not found".to_string()))
    }

    pub fn get_positions_and_symbol_map(
        &self,
        account_id: &str,
    ) -> Result<(Vec<Position>, HashMap<u32, Symbol>), QuestradeAPIError> {
        let positions = self.get_positions(account_id)?;
        let mut symbols = HashMap::new();

        for position in positions.iter() {
            let symbol = self.get_symbol(position.symbol_id)?;
            symbols.entry(symbol.symbol_id).or_insert(symbol);
        }

        Ok((positions, symbols))
    }

    fn make_request(&self, url: &str) -> Result<String, QuestradeAPIError> {
        let resp = self
            .client
            .get(url)
            .bearer_auth(self.token.access_token.clone())
            .send()?;

        if !resp.status().is_success() {
            return Err(QuestradeAPIError::APIError(resp.text()?));
        }

        Ok(resp.text()?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Accounts {
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
pub struct Balances {
    per_currency_balances: Vec<Balance>,
    combined_balances: Vec<Balance>,
}

impl Balances {
    pub fn display_balances(&self) {
        println!(
            "{:<10} | {:<10} | {:<15} | {:>15}",
            "Currency", "Cash", "Market Value", "Total Equity"
        );
        println!("{}", "-".repeat(59));
        for balance in self.per_currency_balances.iter() {
            println!(
                "{:<10} | {:<10.2} | {:<15.2} | {:>15.2}",
                balance.currency, balance.cash, balance.market_value, balance.total_equity
            );
        }

        println!("{}", "=".repeat(59));
        self.combined_balances
            .iter()
            .find(|balance| balance.currency == "CAD")
            .map(|balance| {
                println!(
                    "{:<10} | {:<10.2} | {:<15.2} | {:>15.2}",
                    "Combined", balance.cash, balance.market_value, balance.total_equity
                );
            });

        println!();
    }
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
struct Positions {
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
    pub total_cost: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Symbols {
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

pub fn display_positions_with_dividends(positions: &Vec<Position>, symbols: &HashMap<u32, Symbol>) {
    let title = format!("{}Positions{}", "-".repeat(60), "-".repeat(60));
    println!("{}", title.cyan());
    println!();
    println!(
        "{:<10} | {:<10} | {:<10} | {:<15} | {:<15} | {:<15} | {:<10} | {:<10} | {:>10}",
        "Symbol",
        "Quantity",
        "Avg Price",
        "Book Cost",
        "Market Price",
        "Market Value",
        "Dividend",
        "Yield",
        "P&L"
    );
    println!("{}", "-".repeat(129));

    let mut total_cost = 0.0;
    let mut total_mkt_val = 0.0;

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

        println!(
            "{:<10} | {:<10} | {:<10.2} | {:<15.2} | {:<15.2} | {:<15.2} | {:<10.4} | {:<10.2} | {:>10}",
            position.symbol, quantity, position.average_entry_price, position.total_cost, position.current_price, position.current_market_value, dividend, yield_, colour_pnl(pnl)
        );
    }

    println!("{}", "=".repeat(129));
    println!(
        "{:<10} | {:<10} | {:<10} | {:<15.2} | {:<15} | {:<15.2} | {:<10} | {:<10} | {:>10}",
        "Total",
        "",
        "",
        total_cost,
        "",
        total_mkt_val,
        "",
        "",
        colour_pnl(total_mkt_val - total_cost)
    );
    println!();
}

fn colour_pnl(pnl: f64) -> ColoredString {
    let pnl = (pnl * 100.0).round() / 100.0;

    match 0.0.partial_cmp(&pnl).unwrap() {
        std::cmp::Ordering::Less => pnl.to_string().green(),
        std::cmp::Ordering::Equal => pnl.to_string().normal(),
        std::cmp::Ordering::Greater => pnl.to_string().red(),
    }
}
