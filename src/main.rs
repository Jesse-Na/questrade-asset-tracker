mod questrade_api;

use questrade_api::{get_accounts, get_oauth2_token, get_positions};
use mongodb::{
	bson::{Document, doc},
	sync::{Client, Collection}
};
use dotenv;
use std::env;

fn main() {
    dotenv::dotenv().ok();

    let client = reqwest::blocking::Client::new();
    let coll = get_db_collection();
    let refresh_token = get_refresh_token(&coll);

    let token = match get_oauth2_token(&client, &refresh_token) {
        Ok(token) => {
            println!("Got OAuth2 token: {:?}", token);
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

    for account in accounts {
        match get_positions(&client, &token, &account.id) {
            Ok(positions) => {
                println!("Account: {}", account.id);
                for position in positions {
                    println!(
                        "Symbol: {}, Quantity: {}",
                        position.symbol, position.open_quantity
                    );
                }
            }
            Err(err) => {
                eprintln!(
                    "Error getting positions for account {}: {}",
                    account.id, err
                );
            }
        }
    }
}

fn get_db_collection() -> Collection<Document> {
    let password = env::vars().find(|(key, _)| key == "DB_PASSWORD").unwrap().1;
    let uri = format!("mongodb+srv://user:{password}@cluster0.2dmsm.mongodb.net/?retryWrites=true&w=majority&appName=Cluster0");
    let client = Client::with_uri_str(uri).unwrap();
    let database = client.database("questrade_asset_tracker_db");
    database.collection("refresh_tokens")
}

fn get_refresh_token(coll: &Collection<Document>) -> String {
    String::from(coll.find_one(doc! {})
        .run().unwrap().unwrap().get_str("refresh_token").unwrap())
}

fn update_refresh_token(coll: &Collection<Document>, old_token: &str, new_token: &str) {
    let filter = doc! { "refresh_token": old_token };
    let update = doc! { "$set": { "refresh_token": new_token } };
    coll.update_one(filter, update).run().unwrap();
}
