use std::{collections::HashMap, fmt};

use crate::questrade_api;

#[derive(Eq, Hash, PartialEq)]
enum AssetClass {
    Stocks,
    Bonds,
    Cash,
}

impl From<&AssetClass> for String {
    fn from(asset_class: &AssetClass) -> String {
        String::from(match asset_class {
            AssetClass::Stocks => "Stocks",
            AssetClass::Bonds => "Bonds",
            AssetClass::Cash => "Cash",
        })
    }
}

pub struct Assets {
    total_costs: f64,
    total_market_values: f64,
    total_pnl: f64,
    asset_comp: HashMap<String, f64>,
    simplified_comp: HashMap<AssetClass, f64>,
}

impl Assets {
    pub fn new() -> Assets {
        Assets {
            total_costs: 0.0,
            total_market_values: 0.0,
            total_pnl: 0.0,
            asset_comp: HashMap::new(),
            simplified_comp: HashMap::new(),
        }
    }

    pub fn add_positions(&mut self, positions: &Vec<questrade_api::Position>) {
        for position in positions {
            let mkt_val = position.current_market_value;

            self.total_costs += position.total_cost;
            self.total_market_values += position.current_market_value;
            self.total_pnl += position.open_pnl;

            self.asset_comp.entry(position.symbol.clone()).and_modify(|amt| *amt += mkt_val).or_insert(mkt_val);

            match position.symbol.as_str() {
                "XEQT.TO" | "ZEQT.TO" => {
                    self.simplified_comp.entry(AssetClass::Stocks).and_modify(|amt| *amt += mkt_val).or_insert(mkt_val);
                },
                _ => {
                    self.simplified_comp.entry(AssetClass::Bonds).and_modify(|amt| *amt += mkt_val).or_insert(mkt_val);
                }
            }
        }
    }
}

impl fmt::Display for Assets {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "---Portfolio Summary---\n")?;
        write!(f, "Total Book Costs: {:.2}\nTotal Market Values: {:.2}\nTotal P&L: {:.2}\n", self.total_costs, self.total_market_values, self.total_pnl)?;

        write!(f, "\n{:<10} | {:<10} | {:>10}\n", "Symbol", "Value", "Percent")?;
        write!(f, "{}\n", "-".repeat(36))?;
        for (symbol, value) in &self.asset_comp {
            let percent = value / self.total_market_values * 100.0;
            write!(f, "{:<10} | {:<10.2} | {:>10.2}\n", symbol, value, percent)?;
        }
        write!(f, "{}\n", "=".repeat(36))?;
        write!(f, "{:<10} | {:<10.2}\n", "Total", self.total_market_values)?;

        write!(f, "\n{:<10} | {:<10} | {:>10}\n", "Asset", "Value", "Percent")?;
        write!(f, "{}\n", "-".repeat(36))?;
        for (asset_class, value) in &self.simplified_comp {
            let percent = value / self.total_market_values * 100.0;
            write!(f, "{:<10} | {:<10.2} | {:>10.2}\n", String::from(asset_class), value, percent)?;
        }
        write!(f, "{}\n", "=".repeat(36))?;
        write!(f, "{:<10} | {:<10.2}\n", "Total", self.total_market_values)?;

        Ok(())
    }
}