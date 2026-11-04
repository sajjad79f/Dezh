use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::dto::ErrorDto;

/// خطای استاندارد API.
///
/// هر handler به‌جای ساخت دستی تاپل‌های `(StatusCode, Json(ErrorDto))`،
/// یک `Result<_, ApiError>` برمی‌گرداند و `?` کار تمیزکاری را می‌کند.
/// این همان refactorی است که ۳۷ تکه کد تکراری resolve/خطا را حذف کرد.
pub enum ApiError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    TooManyRequests {
        retry_after_secs: u64,
        message: String,
    },
    Internal(String),
    ServiceUnavailable(String),
}

impl ApiError {
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized(msg.into())
    }

    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::Forbidden(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    pub fn service_unavailable(msg: impl Into<String>) -> Self {
        Self::ServiceUnavailable(msg.into())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::BadRequest(m) => {
                (StatusCode::BAD_REQUEST, Json(ErrorDto { error: m })).into_response()
            }
            ApiError::Unauthorized(m) => {
                (StatusCode::UNAUTHORIZED, Json(ErrorDto { error: m })).into_response()
            }
            ApiError::Forbidden(m) => {
                (StatusCode::FORBIDDEN, Json(ErrorDto { error: m })).into_response()
            }
            ApiError::NotFound(m) => {
                (StatusCode::NOT_FOUND, Json(ErrorDto { error: m })).into_response()
            }
            ApiError::Conflict(m) => {
                (StatusCode::CONFLICT, Json(ErrorDto { error: m })).into_response()
            }
            ApiError::TooManyRequests {
                retry_after_secs,
                message,
            } => (
                StatusCode::TOO_MANY_REQUESTS,
                [(header::RETRY_AFTER, retry_after_secs.to_string())],
                Json(ErrorDto { error: message }),
            )
                .into_response(),
            ApiError::Internal(m) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorDto { error: m }),
            )
                .into_response(),
            ApiError::ServiceUnavailable(m) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorDto { error: m }),
            )
                .into_response(),
        }
    }
}

/// نتیجه‌ی استاندارد همه‌ی handlerها
pub type ApiResult<T> = Result<T, ApiError>;

// ── تبدیل‌های خودکار از خطاهای لایه‌های پایین‌تر ─────────────

impl From<storage::StorageError> for ApiError {
    fn from(e: storage::StorageError) -> Self {
        match e {
            storage::StorageError::NotFound(m) => ApiError::NotFound(m),
            other => ApiError::Internal(other.to_string()),
        }
    }
}

impl From<firewall::FirewallError> for ApiError {
    fn from(e: firewall::FirewallError) -> Self {
        match e {
            firewall::FirewallError::InvalidRule(m) => ApiError::BadRequest(m),
            firewall::FirewallError::RuleNotFound(id) => {
                ApiError::NotFound(format!("rule not found: {id}"))
            }
            firewall::FirewallError::SetNotFound(n) => {
                ApiError::NotFound(format!("address list not found: {n}"))
            }
            firewall::FirewallError::Internal(m) => ApiError::Internal(m),
        }
    }
}

impl From<network::NetworkError> for ApiError {
    fn from(e: network::NetworkError) -> Self {
        match e {
            network::NetworkError::NotFound(m) => ApiError::NotFound(m),
            other => ApiError::Internal(other.to_string()),
        }
    }
}

impl From<routing::RoutingError> for ApiError {
    fn from(e: routing::RoutingError) -> Self {
        match e {
            routing::RoutingError::Invalid(m) => ApiError::BadRequest(m),
            other => ApiError::Internal(other.to_string()),
        }
    }
}