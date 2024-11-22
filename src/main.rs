mod assets;
mod db;
mod questrade_api;

use std::env;
use dotenv;

use assets::Assets;
use db::{get_db_collection, get_refresh_token, update_refresh_token};
use questrade_api::{get_accounts, get_balances, get_oauth2_token, get_positions_and_symbols, display_balances, display_positions_with_dividends};

fn main() {
    dotenv::dotenv().ok();
    let password = env::vars().find(|(key, _)| key == "DB_PASSWORD").unwrap().1;
    let client = reqwest::blocking::Client::new();
    let coll = get_db_collection(&password);
    let refresh_token = get_refresh_token(&coll);

    let token = match get_oauth2_token(&client, &refresh_token) {
        Ok(token) => {
            update_refresh_token(&coll, &refresh_token, &token.refresh_token);
            token
        },
        Err(err) => {
            eprintln!("Error getting OAuth2 token: {}", err);
            return;
        }
    };

    let accounts = match get_accounts(&client, &token) {
        Ok(accounts) => accounts,
        Err(err) => {
            eprintln!("Error getting accounts: {}", err);
            return;
        }
    };

    let mut assets = Assets::new();

    for account in accounts {
        println!("Account: {}—{}", account.type_, account.id);

        let balances = match get_balances(&client, &token, &account.id) {
            Ok(balances) => balances,
            Err(err) => {
                eprintln!("Error getting balances for account {}: {}", account.id, err);
                continue;
            }
        };

        display_balances(&balances);

        let (positions, symbols) = match get_positions_and_symbols(&client, &token, &account.id) {
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
