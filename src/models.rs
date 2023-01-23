use dotenv::dotenv;
use std::env;

pub mod token;
pub mod marketplace;

/// Get database credentials from .env or fail
pub(crate) fn get_database_credentials() -> String {
    dotenv().ok();

    env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file")
}

pub(crate) async fn get_mongo_client() -> mongodb::Client {
    let database_url = get_database_credentials();

    mongodb::Client::with_uri_str(database_url).await.expect("Failed connect to MongoDB")
}