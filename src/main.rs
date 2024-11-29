mod assets;
mod db;
mod questrade_api;

use colored::Colorize;
use dotenv;

use assets::Assets;
use questrade_api::display_positions_with_dividends;

fn main() {
    dotenv::dotenv().ok();

    let client = reqwest::blocking::Client::new();
    let q_api = match questrade_api::QuestradeAPI::new(client) {
        Ok(api) => api,
        Err(err) => {
            eprintln!("Error creating QuestradeAPI: {}", err);
            return;
        }
    };

    let accounts = match q_api.get_accounts() {
        Ok(accounts) => accounts,
        Err(err) => {
            eprintln!("Error getting accounts: {}", err);
            return;
        }
    };

    let mut assets = Assets::new();

    for account in accounts {
        let account_title = format!("Account: {} — {}", account.type_, account.id);
        println!("{}", account_title.blue());

        let balances = match q_api.get_balances(&account.id) {
            Ok(balances) => balances,
            Err(err) => {
                eprintln!("Error getting balances for account {}: {}", account.id, err);
                continue;
            }
        };

        balances.display_balances();

        let (mut positions, symbols) = match q_api.get_positions_and_symbol_map(&account.id) {
            Ok(res) => res,
            Err(err) => {
                eprintln!(
                    "Error getting positions for account {}: {}",
                    account.id, err
                );
                continue;
            }
        };

        assets.add_positions(&positions);
        positions.sort_by(|a, b| b.current_market_value.total_cmp(&a.current_market_value));
        display_positions_with_dividends(&positions, &symbols);
    }

    println!("{}", assets);
}
