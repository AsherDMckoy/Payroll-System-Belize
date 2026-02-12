use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{Json, Request, State},
    http::{
        header::{COOKIE, SET_COOKIE},
        HeaderMap, HeaderValue, StatusCode,
    },
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
    Extension,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool};

const SESSION_COOKIE_NAME: &str = "pms_session";
const SESSION_TTL_SECONDS: i64 = 60 * 60 * 12; // 12 hours

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub id: i32,
    pub username: String,
    pub full_name: String,
    pub role: String,
}

impl AuthenticatedUser {
    pub fn can_manage_payroll(&self) -> bool {
        matches!(self.role.as_str(), "admin" | "payroll")
    }
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub message: String,
    pub username: String,
    pub full_name: String,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct CurrentUserResponse {
    pub username: String,
    pub full_name: String,
    pub role: String,
}

#[derive(FromRow)]
struct UserAuthRow {
    id: i32,
    username: String,
    full_name: String,
    role: String,
    password_hash: String,
    is_active: bool,
}

#[derive(FromRow)]
struct SessionUserRow {
    id: i32,
    username: String,
    full_name: String,
    role: String,
}

pub async fn login(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Response, (StatusCode, String)> {
    let username = req.username.trim();
    let password = req.password.trim();

    if username.is_empty() || password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Username and password are required".to_string(),
        ));
    }

    let user = sqlx::query_as::<_, UserAuthRow>(
        r#"
        SELECT id, username, full_name, role, password_hash, is_active
        FROM app_users
        WHERE username = $1
        LIMIT 1
        "#,
    )
    .bind(username)
    .fetch_optional(&pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    if !user.is_active {
        return Err((
            StatusCode::FORBIDDEN,
            "User account is inactive".to_string(),
        ));
    }

    let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Invalid password hash".to_string(),
        )
    })?;

    if Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_err()
    {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
    }

    let session_token = create_session_token();
    let session_token_hash = sha256_hex(session_token.as_bytes());

    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .map(ToOwned::to_owned);
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .map(ToOwned::to_owned);

    sqlx::query(
        r#"
        INSERT INTO user_sessions (
            user_id,
            session_token_hash,
            expires_at,
            ip_address,
            user_agent
        )
        VALUES ($1, $2, NOW() + ($3 || ' seconds')::interval, $4, $5)
        "#,
    )
    .bind(user.id)
    .bind(session_token_hash)
    .bind(SESSION_TTL_SECONDS)
    .bind(ip_address)
    .bind(user_agent)
    .execute(&pool)
    .await
    .map_err(internal_error)?;

    let payload = Json(LoginResponse {
        message: "Login successful".to_string(),
        username: user.username.clone(),
        full_name: user.full_name.clone(),
        role: user.role.clone(),
    });
    let mut response = payload.into_response();
    response.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_str(&session_cookie_value(&session_token, SESSION_TTL_SECONDS)).map_err(
            |_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to set session cookie".to_string(),
                )
            },
        )?,
    );

    Ok(response)
}

pub async fn logout(
    State(pool): State<PgPool>,
    headers: HeaderMap,
) -> Result<Response, (StatusCode, String)> {
    if let Some(token) = cookie_value(&headers, SESSION_COOKIE_NAME) {
        let token_hash = sha256_hex(token.as_bytes());
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET revoked_at = NOW()
            WHERE session_token_hash = $1
              AND revoked_at IS NULL
            "#,
        )
        .bind(token_hash)
        .execute(&pool)
        .await
        .map_err(internal_error)?;
    }

    let mut response = Json(json!({"message":"Logged out"})).into_response();
    response.headers_mut().insert(
        SET_COOKIE,
        HeaderValue::from_static("pms_session=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0"),
    );
    Ok(response)
}

pub async fn me(
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<CurrentUserResponse>, (StatusCode, String)> {
    Ok(Json(CurrentUserResponse {
        username: user.username,
        full_name: user.full_name,
        role: user.role,
    }))
}

pub async fn require_auth(State(pool): State<PgPool>, mut req: Request, next: Next) -> Response {
    let Some(session_token) = cookie_value(req.headers(), SESSION_COOKIE_NAME) else {
        return unauthenticated_response(req.uri().path());
    };

    let token_hash = sha256_hex(session_token.as_bytes());

    let user = sqlx::query_as::<_, SessionUserRow>(
        r#"
        SELECT u.id, u.username, u.full_name, u.role
        FROM user_sessions s
        INNER JOIN app_users u ON u.id = s.user_id
        WHERE s.session_token_hash = $1
          AND s.revoked_at IS NULL
          AND s.expires_at > NOW()
          AND u.is_active = true
        LIMIT 1
        "#,
    )
    .bind(token_hash)
    .fetch_optional(&pool)
    .await;

    match user {
        Ok(Some(u)) => {
            req.extensions_mut().insert(AuthenticatedUser {
                id: u.id,
                username: u.username,
                full_name: u.full_name,
                role: u.role,
            });
            next.run(req).await
        }
        _ => unauthenticated_response(req.uri().path()),
    }
}

pub async fn ensure_default_admin(pool: &PgPool) -> Result<(), sqlx::Error> {
    let users_table_exists = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM information_schema.tables
            WHERE table_schema = 'public'
              AND table_name = 'app_users'
        )
        "#,
    )
    .fetch_one(pool)
    .await?;

    if !users_table_exists {
        return Ok(());
    }

    let username =
        std::env::var("DEFAULT_ADMIN_USERNAME").unwrap_or_else(|_| "Bully2003".to_string());
    let full_name =
        std::env::var("DEFAULT_ADMIN_FULL_NAME").unwrap_or_else(|_| "Tobi McGuire".to_string());
    let password =
        std::env::var("DEFAULT_ADMIN_PASSWORD").unwrap_or_else(|_| "SpideSenses5!".to_string());

    let existing = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT id
        FROM app_users
        WHERE username = $1
        LIMIT 1
        "#,
    )
    .bind(&username)
    .fetch_optional(pool)
    .await?;

    if existing.is_some() {
        return Ok(());
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| sqlx::Error::Protocol("failed to hash default admin password".into()))?
        .to_string();

    sqlx::query(
        r#"
        INSERT INTO app_users (username, full_name, role, password_hash, is_active)
        VALUES ($1, $2, 'admin', $3, true)
        "#,
    )
    .bind(username)
    .bind(full_name)
    .bind(password_hash)
    .execute(pool)
    .await?;

    Ok(())
}

fn unauthenticated_response(path: &str) -> Response {
    if path.starts_with("/api/") {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error":"Unauthorized"})),
        )
            .into_response()
    } else {
        Redirect::to("/login").into_response()
    }
}

fn create_session_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn sha256_hex(value: &[u8]) -> String {
    let digest = Sha256::digest(value);
    let mut out = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

fn session_cookie_value(token: &str, max_age: i64) -> String {
    let secure = std::env::var("COOKIE_SECURE")
        .ok()
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false);

    let secure_part = if secure { "; Secure" } else { "" };
    format!(
        "{SESSION_COOKIE_NAME}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{secure_part}"
    )
}

fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    let cookie_header = headers.get(COOKIE)?.to_str().ok()?;
    cookie_header.split(';').find_map(|part| {
        let mut split = part.trim().splitn(2, '=');
        let key = split.next()?.trim();
        let value = split.next()?.trim();
        if key == name && !value.is_empty() {
            Some(value.to_string())
        } else {
            None
        }
    })
}

fn internal_error(err: sqlx::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Database error: {err}"),
    )
}
