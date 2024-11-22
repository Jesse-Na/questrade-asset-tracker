use mongodb::{
	bson::{Document, doc},
	sync::{Client, Collection}
};

pub fn get_db_collection(db_password: &str) -> Collection<Document> {
    let uri = format!("mongodb+srv://user:{db_password}@cluster0.2dmsm.mongodb.net/?retryWrites=true&w=majority&appName=Cluster0");
    let client = Client::with_uri_str(uri).unwrap();
    let database = client.database("questrade_asset_tracker_db");
    database.collection("refresh_tokens")
}

pub fn get_refresh_token(coll: &Collection<Document>) -> String {
    String::from(coll.find_one(doc! {})
        .run().unwrap().unwrap().get_str("refresh_token").unwrap())
}

pub fn update_refresh_token(coll: &Collection<Document>, old_token: &str, new_token: &str) {
    let filter = doc! { "refresh_token": old_token };
    let update = doc! { "$set": { "refresh_token": new_token } };
    coll.update_one(filter, update).run().unwrap();
}