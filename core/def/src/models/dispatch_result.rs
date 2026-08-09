#[derive(Debug, Clone)]
pub struct DispatchResult {

    pub handler: String,

    pub success: bool,

    pub error: Option<String>,
}

impl DispatchResult {

    pub fn success(handler: impl Into<String>) -> Self {

        Self {

            handler: handler.into(),

            success: true,

            error: None,
        }
    }

    pub fn failed(

        handler: impl Into<String>,

        error: impl Into<String>,

    ) -> Self {

        Self {

            handler: handler.into(),

            success: false,

            error: Some(error.into()),
        }
    }
}