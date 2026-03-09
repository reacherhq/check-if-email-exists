use crate::config::BackendConfig;
use crate::http::ReacherResponseError;
use bcrypt::verify;
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use warp::http::StatusCode;
use warp::Filter;
use tracing::debug;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

const JWT_SECRET: &[u8] = b"reacher_secret_key_change_me";

pub async fn login_handler(
    pg_pool: PgPool,
    body: LoginRequest,
) -> Result<impl warp::Reply, warp::Rejection> {
    debug!("Login attempt for email: {}", body.email);
    let user = sqlx::query("SELECT password_hash FROM reacher_users WHERE email = $1")
        .bind(&body.email)
        .fetch_optional(&pg_pool)
        .await
        .map_err(|e| {
            debug!("Database error during login: {}", e);
            ReacherResponseError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?
        .ok_or_else(|| {
            debug!("User not found: {}", body.email);
            ReacherResponseError::new(StatusCode::UNAUTHORIZED, "Invalid email or password")
        })?;

    let password_hash: String = sqlx::Row::get(&user, "password_hash");
    debug!("User found, verifying password...");

    // Check password. 
    let is_valid = if password_hash.starts_with("$2") {
        verify(&body.password, &password_hash).unwrap_or_else(|e| {
            debug!("Bcrypt error: {}", e);
            false
        })
    } else {
        // Fallback for initial plain text migration
        body.password == password_hash
    };

    if !is_valid {
        debug!("Invalid password for user: {}", body.email);
        return Err(ReacherResponseError::new(StatusCode::UNAUTHORIZED, "Invalid email or password").into());
    }

    debug!("Password valid, generating token...");

    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() + 60 * 60 * 24; // 24 hours

    let claims = Claims {
        sub: body.email,
        exp: expiration as usize,
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET))
        .map_err(|e| ReacherResponseError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(warp::reply::json(&LoginResponse { token }))
}

pub fn with_db(
    pg_pool: PgPool,
) -> impl Filter<Extract = (PgPool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pg_pool.clone())
}

pub fn auth_filter() -> impl Filter<Extract = (String,), Error = warp::Rejection> + Clone {
    warp::header::<String>("Authorization")
        .and_then(|auth_header: String| async move {
            if !auth_header.starts_with("Bearer ") {
                return Err(warp::reject::custom(ReacherResponseError::new(StatusCode::UNAUTHORIZED, "Invalid authorization header")));
            }

            let token = &auth_header[7..];
            let validation = Validation::default();
            let token_data = jsonwebtoken::decode::<Claims>(
                token,
                &DecodingKey::from_secret(JWT_SECRET),
                &validation,
            ).map_err(|_| warp::reject::custom(ReacherResponseError::new(StatusCode::UNAUTHORIZED, "Invalid token")))?;

            Ok(token_data.claims.sub)
        })
}

pub fn routes(
    pg_pool: Option<PgPool>,
) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
    let pg_pool = match pg_pool {
        Some(pool) => pool,
        None => {
            // This should not happen if bulk is enabled, but let's be safe.
            // We return a filter that always fails if the pool is missing.
            return warp::path!("v1" / "auth" / "login")
                .and_then(|| async { 
                    Err(warp::reject::custom(ReacherResponseError::new(StatusCode::INTERNAL_SERVER_ERROR, "Database pool not initialized")))
                })
                .boxed();
        }
    };

    warp::path!("v1" / "auth" / "login")
        .and(warp::post())
        .and(with_db(pg_pool))
        .and(warp::body::json())
        .and_then(login_handler)
        .boxed()
}
