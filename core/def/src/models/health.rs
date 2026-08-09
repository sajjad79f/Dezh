#[derive(Debug, Clone)]

pub enum HealthStatus {

    Healthy,

    Degraded,

    Unhealthy,
}

#[derive(Debug, Clone)]

pub struct Health {

    pub status: HealthStatus,

    pub message: String,
}

impl Health {

    pub fn healthy() -> Self {

        Self {

            status: HealthStatus::Healthy,

            message: String::from("OK"),
        }
    }

    pub fn degraded(message: impl Into<String>) -> Self {

        Self {

            status: HealthStatus::Degraded,

            message: message.into(),
        }
    }

    pub fn unhealthy(message: impl Into<String>) -> Self {

        Self {

            status: HealthStatus::Unhealthy,

            message: message.into(),
        }
    }
}