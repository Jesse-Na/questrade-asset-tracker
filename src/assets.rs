use crate::asset_tracker;
use colored::{Color, ColoredString, Colorize};
use std::{collections::HashMap, fmt};

const STOCK_TARGET: f64 = 50.0;
const BOND_TARGET: f64 = 50.0;
const CASH_TARGET: f64 = 0.0;
const MARGIN_OF_WARNING: f64 = 2.5;
const MARGIN_OF_ERROR: f64 = 5.0;

#[derive(Eq, Hash, PartialEq, Clone)]
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
    asset_to_class_map: HashMap<String, AssetClass>,
    class_to_colour_map: HashMap<AssetClass, Color>,
    asset_map: HashMap<String, (f64, f64)>,
    class_map: HashMap<AssetClass, (f64, f64)>,
}

impl Assets {
    pub fn new() -> Assets {
        let mut asset_class_map = HashMap::new();
        asset_class_map.insert("XEQT.TO".to_string(), AssetClass::Stocks);
        asset_class_map.insert("ZEQT.TO".to_string(), AssetClass::Stocks);
        asset_class_map.insert("ZAG.TO".to_string(), AssetClass::Bonds);

        let mut asset_colour_map = HashMap::new();
        asset_colour_map.insert(
            AssetClass::Stocks,
            Color::TrueColor {
                r: 245,
                g: 169,
                b: 184,
            },
        );
        asset_colour_map.insert(
            AssetClass::Bonds,
            Color::TrueColor {
                r: 91,
                g: 206,
                b: 250,
            },
        );
        asset_colour_map.insert(
            AssetClass::Cash,
            Color::TrueColor {
                r: 186,
                g: 218,
                b: 85,
            },
        );

        Assets {
            total_costs: 0.0,
            total_market_values: 0.0,
            asset_to_class_map: asset_class_map,
            class_to_colour_map: asset_colour_map,
            asset_map: HashMap::new(),
            class_map: HashMap::new(),
        }
    }

    pub fn add_positions(&mut self, positions: &Vec<asset_tracker::Position>) {
        for position in positions {
            let book_cost = position.total_cost;
            let mkt_val = position.current_market_value;

            self.total_costs += position.total_cost;
            self.total_market_values += position.current_market_value;

            self.asset_map
                .entry(position.symbol.clone())
                .and_modify(|(cost, val)| {
                    *cost += book_cost;
                    *val += mkt_val;
                })
                .or_insert((book_cost, mkt_val));

            let asset_class = self
                .asset_to_class_map
                .get(&position.symbol)
                .unwrap_or(&AssetClass::Cash);

            self.class_map
                .entry(asset_class.clone())
                .and_modify(|(cost, val)| {
                    *cost += book_cost;
                    *val += mkt_val;
                })
                .or_insert((book_cost, mkt_val));
        }
    }

    fn colour_symbol(&self, symbol: &String) -> ColoredString {
        let colour = match self.asset_to_class_map.get(symbol) {
            Some(asset_class) => self.class_to_colour_map.get(asset_class),
            None => self.class_to_colour_map.get(&AssetClass::Cash),
        };

        match colour {
            Some(&colour) => symbol.color(colour),
            None => symbol.normal(),
        }
    }

    fn colour_asset(&self, asset_class: &AssetClass) -> ColoredString {
        match self.class_to_colour_map.get(asset_class) {
            Some(&colour) => String::from(asset_class).color(colour),
            None => String::from(asset_class).normal(),
        }
    }

    fn colour_percent(&self, percent: f64, asset_class: &AssetClass) -> ColoredString {
        let percent = (percent * 100.0).round() / 100.0;

        let diff = match asset_class {
            AssetClass::Stocks => STOCK_TARGET - percent,
            AssetClass::Bonds => BOND_TARGET - percent,
            AssetClass::Cash => CASH_TARGET - percent,
        };

        match diff.abs() {
            x if x < MARGIN_OF_WARNING => percent.to_string().green(),
            x if x >= MARGIN_OF_ERROR => percent.to_string().red(),
            _ => percent.to_string().yellow(),
        }
    }

    fn get_asset_comp(&self) -> Vec<(String, f64, f64)> {
        let mut asset_comp: Vec<_> = self
            .asset_map
            .iter()
            .map(|(symbol, (cost, val))| (symbol.clone(), *cost, *val))
            .collect();

        asset_comp.sort_by(|a, b| b.2.total_cmp(&a.2));
        asset_comp
    }

    fn get_simplified_comp(&self) -> Vec<(AssetClass, f64, f64)> {
        let mut simplified_comp: Vec<_> = self
            .class_map
            .iter()
            .map(|(asset_class, (cost, val))| (asset_class.clone(), *cost, *val))
            .collect();

        simplified_comp.sort_by(|a, b| b.2.total_cmp(&a.2));
        simplified_comp
    }

    fn display_asset_comp(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let header = format!(
            "\n{:<10} | {:<15} | {:<15} | {:>10}\n",
            "Symbol", "Book Cost", "Market Value", "Percent"
        );
        write!(f, "{}", header)?;
        write!(f, "{}\n", "-".repeat(59))?;

        for (symbol, book_cost, mkt_val) in &self.get_asset_comp() {
            let percent = mkt_val / self.total_market_values * 100.0;
            write!(
                f,
                "{:<10} | {:<15.2} | {:<15.2} | {:>10.2}\n",
                self.colour_symbol(symbol),
                book_cost,
                mkt_val,
                percent
            )?;
        }
        write!(f, "{}\n", "=".repeat(59))?;
        write!(
            f,
            "{:<10} | {:<15.2} | {:<15.2}\n",
            "Total", self.total_costs, self.total_market_values
        )?;

        Ok(())
    }

    fn display_simplified_comp(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let header = format!(
            "\n{:<10} | {:<15} | {:<15} | {:>10}\n",
            "Asset", "Book Cost", "Market Value", "Percent"
        );
        write!(f, "{}", header)?;
        write!(f, "{}\n", "-".repeat(59))?;

        for (asset_class, book_cost, mkt_val) in &self.get_simplified_comp() {
            let percent = mkt_val / self.total_market_values * 100.0;
            write!(
                f,
                "{:<10} | {:<15.2} | {:<15.2} | {:>10}\n",
                self.colour_asset(asset_class),
                book_cost,
                mkt_val,
                self.colour_percent(percent, asset_class)
            )?;
        }
        write!(f, "{}\n", "=".repeat(59))?;
        write!(
            f,
            "{:<10} | {:<15.2} | {:<15.2}\n",
            "Total", self.total_costs, self.total_market_values
        )?;

        Ok(())
    }
}

impl fmt::Display for Assets {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let title = format!("{}Portfolio Summary{}", "-".repeat(21), "-".repeat(21));
        write!(f, "{}\n", title.cyan())?;
        self.display_asset_comp(f)?;
        self.display_simplified_comp(f)?;

        Ok(())
    }
}
