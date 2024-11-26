mod assets;
mod db;
mod questrade_api;

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
        println!("Account: {} — {}", account.type_, account.id);

        let balances = match q_api.get_balances(&account.id) {
            Ok(balances) => balances,
            Err(err) => {
                eprintln!("Error getting balances for account {}: {}", account.id, err);
                continue;
            }
        };

        balances.display_balances();

        let (positions, symbols) = match q_api.get_positions_and_symbol_map(&account.id) {
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
        display_positions_with_dividends(&positions, &symbols);
    }

    println!("{}", assets);
}
