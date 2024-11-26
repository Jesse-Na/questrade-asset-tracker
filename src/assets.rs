use std::{collections::HashMap, fmt};

use colored::{Color, ColoredString, Colorize};

use crate::questrade_api;

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
    asset_class_map: HashMap<String, AssetClass>,
    asset_colour_map: HashMap<AssetClass, Color>,
    asset_comp: HashMap<String, (f64, f64)>,
    simplified_comp: HashMap<AssetClass, (f64, f64)>,
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
            asset_class_map,
            asset_colour_map,
            asset_comp: HashMap::new(),
            simplified_comp: HashMap::new(),
        }
    }

    pub fn add_positions(&mut self, positions: &Vec<questrade_api::Position>) {
        for position in positions {
            let book_cost = position.total_cost;
            let mkt_val = position.current_market_value;

            self.total_costs += position.total_cost;
            self.total_market_values += position.current_market_value;

            self.asset_comp
                .entry(position.symbol.clone())
                .and_modify(|(cost, val)| {
                    *cost += book_cost;
                    *val += mkt_val;
                })
                .or_insert((book_cost, mkt_val));

            let asset_class = match self.asset_class_map.get(&position.symbol) {
                Some(asset_class) => asset_class,
                None => &AssetClass::Cash,
            };

            self.simplified_comp
                .entry(asset_class.clone())
                .and_modify(|(cost, val)| {
                    *cost += book_cost;
                    *val += mkt_val;
                })
                .or_insert((book_cost, mkt_val));
        }
    }

    fn colour_symbol(&self, symbol: &String) -> ColoredString {
        let colour = match self.asset_class_map.get(symbol) {
            Some(asset_class) => self.asset_colour_map.get(asset_class),
            None => self.asset_colour_map.get(&AssetClass::Cash),
        };

        match colour {
            Some(&colour) => symbol.color(colour),
            None => symbol.normal(),
        }
    }

    fn colour_asset(&self, asset_class: &AssetClass) -> ColoredString {
        match self.asset_colour_map.get(asset_class) {
            Some(&colour) => String::from(asset_class).color(colour),
            None => String::from(asset_class).normal(),
        }
    }

    fn display_asset_comp(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let header = format!(
            "\n{:<10} | {:<15} | {:<15} | {:>10}\n",
            "Symbol", "Book Cost", "Market Value", "Percent"
        );
        write!(f, "{}", header)?;
        write!(f, "{}\n", "-".repeat(59))?;

        for (symbol, (book_cost, mkt_val)) in &self.asset_comp {
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

        for (asset_class, (book_cost, mkt_val)) in &self.simplified_comp {
            let percent = mkt_val / self.total_market_values * 100.0;
            write!(
                f,
                "{:<10} | {:<15.2} | {:<15.2} | {:>10.2}\n",
                self.colour_asset(asset_class),
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
}

impl fmt::Display for Assets {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "---Portfolio Summary---\n")?;
        self.display_asset_comp(f)?;
        self.display_simplified_comp(f)?;

        Ok(())
    }
}
