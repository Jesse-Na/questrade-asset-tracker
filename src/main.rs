mod assets;
mod db;
mod questrade_api;

use assets::Assets;
use colored::Colorize;
use db::DatabaseAPI;
use questrade_api::display_positions_with_dividends;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Questrade Asset Tracker",
    about = "Track your Questrade assets"
)]
struct Opt {
    #[structopt(long = "auth")]
    authorization_token: Option<String>,
}

#[tokio::main]
async fn main() {
    let db = match DatabaseAPI::new().await {
        Ok(db) => db,
        Err(err) => {
            eprintln!("Error creating DatabaseAPI: {}", err);
            return;
        }
    };

    let opt = Opt::from_args();
    if let Some(token) = opt.authorization_token {
        match db.insert_refresh_token(&token).await {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error inserting refresh token: {}", err);
                return;
            }
        }
    }

    let client = reqwest::Client::new();
    let q_api = match questrade_api::QuestradeAPI::new(client, db).await {
        Ok(api) => api,
        Err(err) => {
            eprintln!("Error creating QuestradeAPI: {}", err);
            return;
        }
    };

    let accounts = match q_api.get_accounts().await {
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

        let balances = match q_api.get_balances(&account.id).await {
            Ok(balances) => balances,
            Err(err) => {
                eprintln!("Error getting balances for account {}: {}", account.id, err);
                continue;
            }
        };

        balances.display_balances();

        let (mut positions, symbols) = match q_api.get_positions_and_symbol_map(&account.id).await {
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
