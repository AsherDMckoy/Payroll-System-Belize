pub mod db;
pub mod handlers;
pub mod models;
use std::net::SocketAddr;

use crate::handlers::api::{dashboard_overview, employees_list, payroll_breakdown};
use crate::handlers::payroll::{generate_payroll, payroll_period_details, payroll_periods};
use crate::handlers::reports::{generate_report, report_filter_options};
use crate::models::views::{HtmlTemplate, IndexTemplate};
use axum::{
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use models::views::Error404Template;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenvy::dotenv().ok();
    env_logger::init();

    println!("Connecting to database...");
    let pool = db::create_pool()
        .await
        .expect("Failed to create database pool");
    println!("✓ Database connected!");
    db::run_migrations(&pool)
        .await
        .expect("Failed to run database migrations");
    println!("✓ Migrations complete!");

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/payroll", get(index_handler))
        .route("/employees", get(index_handler))
        .route("/reports", get(index_handler))
        .route("/api/dashboard/overview", get(dashboard_overview))
        .route("/api/employees", get(employees_list))
        .route("/api/payroll-breakdown", get(payroll_breakdown))
        .route("/api/payroll/periods", get(payroll_periods))
        .route("/api/payroll/period-details", get(payroll_period_details))
        .route("/api/payroll/generate", post(generate_payroll))
        .route("/api/reports/options", get(report_filter_options))
        .route("/api/reports/generate", post(generate_report))
        .route_service("/favicon.ico", ServeFile::new("assets/favicon.ico"))
        .nest_service("/assets", ServeDir::new("assets"))
        .fallback(fallback_handler)
        .with_state(pool);

    // run our app with hyper
    let host = std::env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(9000);
    let addr: SocketAddr = format!("{host}:{port}").parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Listening on {}:{}", host, port);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn index_handler() -> impl IntoResponse {
    let index = IndexTemplate {
        name: "Morrigan Aensland".to_string(),
    };
    HtmlTemplate(index)
}

// async fn test() -> Html<&'static str> {
//     Html("<h1>Hello, World: Super like my man cat local time 899!</h1>")
// }

async fn fallback_handler() -> impl IntoResponse {
    let not_found = Error404Template;
    HtmlTemplate(not_found)
}
