pub mod db;
pub mod handlers;
pub mod models;
use std::net::SocketAddr;

use crate::handlers::api::{dashboard_overview, employees_list, payroll_breakdown};
use crate::handlers::auth::{
    ensure_default_admin, login, logout, me as auth_me, require_auth, AuthenticatedUser,
};
use crate::handlers::payroll::{
    approve_payroll, execute_payroll, generate_payroll, payroll_audit_trail,
    payroll_period_details, payroll_periods,
};
use crate::handlers::reports::{generate_report, report_filter_options};
use crate::models::views::{HtmlTemplate, IndexTemplate, LoginTemplate};
use axum::{
    extract::State,
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use models::views::Error404Template;
use serde_json::json;
use sqlx::postgres::PgPool;
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
    let run_migrations_on_startup = std::env::var("RUN_MIGRATIONS_ON_STARTUP")
        .ok()
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(true);
    if run_migrations_on_startup {
        db::run_migrations(&pool)
            .await
            .expect("Failed to run database migrations");
        println!("✓ Migrations complete!");
    }

    if let Err(err) = ensure_default_admin(&pool).await {
        log::error!("Failed to ensure default admin user: {err}");
    }

    let protected_routes = Router::new()
        .route("/", get(index_handler))
        .route("/payroll", get(index_handler))
        .route("/employees", get(index_handler))
        .route("/reports", get(index_handler))
        .route("/api/auth/me", get(auth_me))
        .route("/api/dashboard/overview", get(dashboard_overview))
        .route("/api/employees", get(employees_list))
        .route("/api/payroll-breakdown", get(payroll_breakdown))
        .route("/api/payroll/periods", get(payroll_periods))
        .route("/api/payroll/period-details", get(payroll_period_details))
        .route("/api/payroll/audit", get(payroll_audit_trail))
        .route("/api/payroll/generate", post(generate_payroll))
        .route("/api/payroll/approve", post(approve_payroll))
        .route("/api/payroll/execute", post(execute_payroll))
        .route("/api/reports/options", get(report_filter_options))
        .route("/api/reports/generate", post(generate_report))
        .route_layer(middleware::from_fn_with_state(pool.clone(), require_auth));

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .route("/login", get(login_page))
        .route("/api/auth/login", post(login))
        .route("/api/auth/logout", post(logout))
        .merge(protected_routes)
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
    let addr: SocketAddr = format!("{host}:{port}").parse().map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid bind address {host}:{port}: {err}"),
        )
    })?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Listening on {}:{}", host, port);

    axum::serve(listener, app).await?;
    Ok(())
}

async fn index_handler(Extension(user): Extension<AuthenticatedUser>) -> impl IntoResponse {
    let index = IndexTemplate {
        name: user.full_name,
    };
    HtmlTemplate(index)
}

async fn login_page() -> impl IntoResponse {
    HtmlTemplate(LoginTemplate)
}

async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({"status":"ok"})))
}

async fn ready_handler(State(pool): State<PgPool>) -> impl IntoResponse {
    match sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&pool)
        .await
    {
        Ok(_) => (StatusCode::OK, Json(json!({"status":"ready"}))),
        Err(err) => {
            log::error!("Readiness check failed: {err}");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({"status":"not_ready"})),
            )
        }
    }
}

// async fn test() -> Html<&'static str> {
//     Html("<h1>Hello, World: Super like my man cat local time 899!</h1>")
// }

async fn fallback_handler() -> impl IntoResponse {
    let not_found = Error404Template;
    HtmlTemplate(not_found)
}
