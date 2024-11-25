use mongodb::{
    bson::{doc, Document},
    sync::{Client, Collection},
};

pub fn get_db_collection(db_password: &str) -> Result<Collection<Document>, mongodb::error::Error> {
    let uri = format!("mongodb+srv://user:{db_password}@cluster0.2dmsm.mongodb.net/?retryWrites=true&w=majority&appName=Cluster0");
    let client = Client::with_uri_str(uri)?;
    let database = client.database("questrade_asset_tracker_db");
    Ok(database.collection("refresh_tokens"))
}

pub fn get_refresh_token(
    coll: &Collection<Document>,
) -> Result<Option<String>, mongodb::error::Error> {
    if let Some(doc) = coll.find_one(doc! {}).run()? {
        if let Ok(token) = doc.get_str("refresh_token") {
            return Ok(Some(String::from(token)));
        }
    }

    Ok(None)
}

pub fn update_refresh_token(
    coll: &Collection<Document>,
    old_token: &str,
    new_token: &str,
) -> Result<(), mongodb::error::Error> {
    let filter = doc! { "refresh_token": old_token };
    let update = doc! { "$set": { "refresh_token": new_token } };
    coll.update_one(filter, update).run()?;

    Ok(())
}
