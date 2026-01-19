pub mod models;
use std::net::SocketAddr;

use crate::models::views::{HtmlTemplate, IndexTemplate};
use axum::{response::IntoResponse, routing::get, Router};
use models::views::Error404Template;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenvy::dotenv().ok();
    env_logger::init();
    // build our application with a single route
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/payroll", get(index_handler))
        .route("/employees", get(index_handler))
        .route("/reports", get(index_handler))
        .route_service("/favicon.ico", ServeFile::new("assets/favicon.ico"))
        .nest_service("/assets", ServeDir::new("assets"))
        .fallback(fallback_handler);

    // run our app with hyper, listening globally on port 9000
    let port = 9000;
    let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Listening at localhost on port {}", port);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn index_handler() -> impl IntoResponse {
    let index = IndexTemplate {
        name: "Morrigan Aensland".to_string(),
    };
    HtmlTemplate(index)
}

async fn fallback_handler() -> impl IntoResponse {
    let not_found = Error404Template;
    HtmlTemplate(not_found)
}
