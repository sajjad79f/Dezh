use axum::{
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
};

//use shared_kernel::prelude::ServiceContainer;
use storage::{DbPool, SessionRepo, UserRow};

use crate::state::AppState;

pub struct AuthUser(pub UserRow);

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = extract_bearer(&parts.headers)
            .ok_or((StatusCode::UNAUTHORIZED, "missing token".into()))?;

        let pool = state
            .services
            .resolve::<DbPool>()
            .ok_or((StatusCode::SERVICE_UNAVAILABLE, "database unavailable".into()))?;

        let repo = SessionRepo::new(pool.inner());
        let user = repo
            .find_user_by_token(&token)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .ok_or((StatusCode::UNAUTHORIZED, "invalid or expired token".into()))?;

        Ok(AuthUser(user))
    }
}

pub fn extract_bearer(headers: &axum::http::HeaderMap) -> Option<String> {
    let value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    let token = value.strip_prefix("Bearer ").or_else(|| value.strip_prefix("bearer "))?;
    Some(token.trim().to_string())
}

pub fn generate_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}