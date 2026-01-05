use serde::Serialize;

#[derive(Serialize)]
pub struct R<T> {
    pub code: u16,
    pub message: String,
    pub data: Option<T>,
}

impl<T> R<T> {
    pub fn ok(data: T) -> Self {
        Self {
            code: 0,
            message: "ok".into(),
            data: Some(data),
        }
    }

    pub fn error(message: impl Into<String>, code: u16) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }
}
