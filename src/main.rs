mod asset_tracker;
mod assets;
mod db;
mod questrade_api;

use db::DatabaseAPI;
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

    let questrade_api = match questrade_api::QuestradeAPI::new(db).await {
        Ok(api) => api,
        Err(err) => {
            eprintln!("Error creating QuestradeAPI client: {}", err);
            return;
        }
    };

    let asset_tracker = match asset_tracker::AssetTracker::new(questrade_api).await {
        Ok(api) => api,
        Err(err) => {
            eprintln!("Error starting Asset Tracker: {}", err);
            return;
        }
    };

    println!("Welcome to the Questrade Asset Tracker!");
    println!("You can quit at anytime by pressing Ctrl+C or supplying the `quit` command");
    display_help();

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        match input {
            "quit" => break,
            "help" => display_help(),
            "home" => asset_tracker.display_home(),
            "accounts" => asset_tracker.display_accounts(),
            "positions" => asset_tracker.display_positions_with_dividends(None),
            "summary" => asset_tracker.display_summary(),
            _ => println!("Invalid command. Please try again."),
        }
    }
}

fn display_help() {
    println!("Below is a list of commands and their arguments:");
    println!();
    println!("`quit` — Quit the program");
    println!("`help` — Display these instructions again");
    println!("`home` — Display the home dashboard");
    println!("`accounts` — Display all accounts and their balances");
    println!("`positions` — Display all positions and their dividends");
    println!("`summary` — Display a high-level summary of your portfolio");
}
