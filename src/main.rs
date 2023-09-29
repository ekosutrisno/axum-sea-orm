use dotenv::dotenv;
use sea_orm::Database;

mod controller;
mod model;
mod schema;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    run(database_url).await
}

pub async fn run(database_uri: String) {
    let database = Database::connect(database_uri).await.unwrap();
    let app = controller::create_routes(database).await;

    println!("Listening {:<12}", 8000);
    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
