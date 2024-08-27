use jmespath::JmespathError;
use sled::Error as SledError;
use polars::error::PolarsError;
use openai_api_rs::v1::error::APIError as OpenAIApiError;


#[derive(Debug)]
pub enum ErrorType {
    GeneralError,
    JmespathError,
    SledError,
    CacheError,
    OpenAIApiError,
    PolarsError
}

#[derive(Debug)]
pub struct CustomError {
    pub message: String,
    pub kind: ErrorType,
}

impl CustomError {
    pub fn general_error<S: AsRef<str>>(message: S) -> Self {
        Self {
            message: message.as_ref().to_string(),
            kind: ErrorType::GeneralError
        }
    }

    pub fn cache_error<S: AsRef<str>>(message: S) -> Self {
        Self {
            message: message.as_ref().to_string(),
            kind: ErrorType::CacheError
        }
    }

    pub fn sled_error<S: AsRef<str>>(message: S) -> Self {
        Self {
            message: message.as_ref().to_string(),
            kind: ErrorType::SledError
        }
    }
}


impl From<JmespathError> for CustomError {
    fn from(err: JmespathError) -> Self {
        Self {
            message: format!("{}", err),
            kind: ErrorType::JmespathError,
        }
    }
}
impl From<SledError> for CustomError {
    fn from(err: SledError) -> Self {
        Self {
            message: format!("{}", err),
            kind: ErrorType::SledError,
        }
    }
}
impl From<OpenAIApiError> for CustomError {
    fn from(err: OpenAIApiError) -> Self {
        Self {
            message: format!("{}", err),
            kind: ErrorType::OpenAIApiError,
        }
    }
}
impl From<PolarsError> for CustomError {
    fn from(err: PolarsError) -> Self {
        Self {
            message: format!("{:?}", err),
            kind: ErrorType::PolarsError,
        }
    }
}