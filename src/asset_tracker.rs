use crate::{
    assets::Assets,
    questrade_api::{QuestradeAPI, QuestradeAPIError},
};
use colored::{ColoredString, Colorize};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type AccountID = String;
type SymbolID = u32;

pub struct AssetTracker {
    questrade_api: QuestradeAPI,
    accounts: Vec<Account>,
    assets: Assets,
    positions: HashMap<AccountID, Vec<Position>>,
    balances: HashMap<AccountID, Balances>,
    symbols: HashMap<SymbolID, Symbol>,
}

impl AssetTracker {
    pub async fn new(questrade_api: QuestradeAPI) -> Result<Self, QuestradeAPIError> {
        let resp = questrade_api
            .make_request(String::from("v1/accounts"))
            .await?;
        let accounts = serde_json::from_str::<Accounts>(&resp)?.accounts;
        let mut assets = Assets::new();
        let mut balances = HashMap::new();
        let mut positions = HashMap::new();
        let mut symbols = HashMap::new();

        for account in accounts.iter() {
            let resp = questrade_api
                .make_request(format!("v1/accounts/{}/balances", account.id))
                .await?;
            balances.insert(account.id.clone(), serde_json::from_str::<Balances>(&resp)?);

            let resp = questrade_api
                .make_request(format!("v1/accounts/{}/positions", account.id))
                .await?;
            let acct_positions = serde_json::from_str::<Positions>(&resp)?.positions;

            for position in acct_positions.iter() {
                let resp = questrade_api
                    .make_request(format!("v1/symbols/{}", position.symbol_id))
                    .await?;
                let symbol = serde_json::from_str::<Symbols>(&resp)?;

                if let Some(symbol) = symbol.symbols.first() {
                    symbols.insert(symbol.symbol_id, symbol.clone());
                }
            }

            assets.add_positions(&acct_positions);
            positions.insert(account.id.clone(), acct_positions);
        }

        Ok(Self {
            questrade_api,
            accounts,
            assets,
            positions,
            balances,
            symbols,
        })
    }

    pub fn display_accounts(&self) {
        for account in self.accounts.iter() {
            println!("{}", account);

            if let Some(balances) = self.balances.get(&account.id) {
                balances.display_balances();
            } else {
                println!("No balances")
            }
        }
    }

    pub fn display_home(&self) {
        for account in self.accounts.iter() {
            println!("{}", account);

            if let Some(balances) = self.balances.get(&account.id) {
                balances.display_balances();
            } else {
                println!("No balances")
            }

            if let Some(_) = self.positions.get(&account.id) {
                self.display_positions_with_dividends(Some(&account.id));
            } else {
                println!("No positions")
            }
        }

        self.display_summary();
    }

    pub fn display_positions_with_dividends(&self, account_id: Option<&str>) {
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

        let positions = match account_id {
            Some(account_id) => self.positions.get(account_id).unwrap(),
            None => &self.positions.values().flatten().cloned().collect(),
        };

        for position in positions {
            let (dividend, yield_) = if let Some(symbol) = self.symbols.get(&position.symbol_id) {
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
                position.symbol, quantity, position.average_entry_price, position.total_cost, position.current_price, position.current_market_value, dividend, yield_, self.colour_pnl(pnl)
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
            self.colour_pnl(total_mkt_val - total_cost)
        );
        println!();
    }

    pub fn display_summary(&self) {
        println!("{}", self.assets);
    }

    fn colour_pnl(&self, pnl: f64) -> ColoredString {
        let pnl = (pnl * 100.0).round() / 100.0;

        match 0.0.partial_cmp(&pnl).unwrap() {
            std::cmp::Ordering::Less => pnl.to_string().green(),
            std::cmp::Ordering::Equal => pnl.to_string().normal(),
            std::cmp::Ordering::Greater => pnl.to_string().red(),
        }
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
    pub id: AccountID,
}

impl std::fmt::Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let account_title = format!("Account: {} — {}", self.type_, self.id);
        write!(f, "{}", account_title.blue())
    }
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
            "Currency", "Cash", "Market Equity", "Total Equity"
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
struct Positions {
    positions: Vec<Position>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub symbol: String,
    pub symbol_id: SymbolID,
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
    pub symbol_id: SymbolID,
    pub dividend: f64,
    pub yield_: f64,
}
