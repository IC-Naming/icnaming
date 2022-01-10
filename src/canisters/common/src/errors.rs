use candid::{CandidType, Deserialize};
use thiserror::Error;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, CandidType, Deserialize, Error)]
pub enum ICNSError {
    #[error("there is a unknown error raised")]
    Unknown,
    #[error("error from remote, {0:?}")]
    RemoteError(ErrorInfo),
    #[error("the canister name is not allow")]
    InvalidCanisterName,
    #[error("caller not changed since you are not the owner")]
    OwnerOnly,
    #[error("owner is invalid")]
    InvalidOwner,
    #[error("name is invalid, reason: {reason:?}")]
    InvalidName { reason: String },
    #[error("name is unavailable, reason: {reason:?}")]
    NameUnavailable { reason: String },
    #[error("permission deny")]
    PermissionDenied,
    #[error("Registration has been taken")]
    RegistrationHasBeenTaken,
    #[error("Registration is not found")]
    RegistrationNotFound,
    #[error("Top level named had been set")]
    TopNameAlreadyExists,
    #[error("registry for {name:?} is not found")]
    RegistryNotFoundError { name: String },
    #[error("resolver for {name:?} is not found")]
    ResolverNotFoundError { name: String },
    #[error("operator should not be the same to the owner")]
    OperatorShouldNotBeTheSameToOwner,
    #[error("year must be in rang [{min:?},{max:?})")]
    YearsRangeError { min: u64, max: u64 },
    #[error("invalid resolver key: {key:?}")]
    InvalidResolverKey { key: String },
    #[error("Length of value must be less than {max:?}")]
    ValueMaxLengthError { max: usize },
    #[error("Length of {field:?} must be in range [{min:?}, {max:?})")]
    ValueShouldBeInRangeError {
        field: String,
        min: usize,
        max: usize,
    },
    #[error("You have reached the maximum number of favorites: {max:?}")]
    TooManyFavorites { max: usize },
    #[error("Unauthorized, please login first")]
    Unauthorized,
}

impl ICNSError {
    pub(crate) fn code(&self) -> u32 {
        match self {
            ICNSError::Unknown => 1,
            ICNSError::RemoteError(_) => 2,
            ICNSError::InvalidCanisterName => 3,
            ICNSError::InvalidOwner => 4,
            ICNSError::OwnerOnly => 5,
            ICNSError::InvalidName { .. } => 6,
            ICNSError::NameUnavailable { .. } => 7,
            ICNSError::PermissionDenied => 8,
            ICNSError::RegistrationHasBeenTaken => 9,
            ICNSError::RegistrationNotFound => 10,
            ICNSError::TopNameAlreadyExists => 11,
            ICNSError::RegistryNotFoundError { .. } => 12,
            ICNSError::ResolverNotFoundError { .. } => 13,
            ICNSError::OperatorShouldNotBeTheSameToOwner => 14,
            ICNSError::YearsRangeError { .. } => 15,
            ICNSError::InvalidResolverKey { .. } => 16,
            ICNSError::ValueMaxLengthError { .. } => 17,
            ICNSError::ValueShouldBeInRangeError { .. } => 18,
            ICNSError::TooManyFavorites { .. } => 19,
            ICNSError::Unauthorized => 20,
        }
    }
}

/// Error information
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, CandidType, Deserialize)]
pub struct ErrorInfo {
    /// Error code
    code: u32,
    /// Error message
    message: String,
}

pub fn get_error_code(error: ICNSError) -> ErrorInfo {
    ErrorInfo {
        code: error.code(),
        message: error.to_string(),
    }
}

pub type ICNSResult<T> = anyhow::Result<T, ICNSError>;

/// A helper function to convert anyhow::Result<T, ICNSError> to ICNSResult<T>
pub type ICNSActorResult<T> = Result<T, ErrorInfo>;

impl From<ICNSError> for ErrorInfo {
    fn from(error: ICNSError) -> Self {
        get_error_code(error)
    }
}

impl From<ErrorInfo> for ICNSError {
    fn from(error: ErrorInfo) -> Self {
        ICNSError::RemoteError(error)
    }
}

pub fn to_actor_result<T>(result: ICNSResult<T>) -> ICNSActorResult<T> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => Err(get_error_code(error)),
    }
}
