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
    YearsRangeError { min: u32, max: u32 },
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
    #[error("invalid quota order details")]
    InvalidQuotaOrderDetails,
    #[error("please finish the previous order first")]
    PendingOrder,
    #[error("quota order is not found")]
    OrderNotFound,
    #[error("refund failed, please try again later")]
    RefundFailed,
    #[error("too many operators")]
    OperatorCountExceeded,
    #[error("canister call error, rejected by {rejection_code:?}")]
    CanisterCallError {
        rejection_code: String,
        message: String,
    },
    #[error("invalid resolver value format for {value:?}, it should be formatted as {format:?}")]
    InvalidResolverValueFormat { value: String, format: String },
    #[error("Some operations are processing, please try again later")]
    Conflict,
    #[error("Insufficient quota")]
    InsufficientQuota,
    #[error("System is maintaining, please try again later")]
    SystemMaintaining,
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
            ICNSError::InvalidQuotaOrderDetails => 21,
            ICNSError::PendingOrder => 22,
            ICNSError::OrderNotFound => 23,
            ICNSError::RefundFailed => 24,
            ICNSError::OperatorCountExceeded => 25,
            ICNSError::CanisterCallError { .. } => 26,
            ICNSError::InvalidResolverValueFormat { .. } => 27,
            ICNSError::Conflict => 28,
            ICNSError::InsufficientQuota => 29,
            ICNSError::SystemMaintaining => 30,
        }
    }
}

/// Error information
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, CandidType, Deserialize)]
pub struct ErrorInfo {
    /// Error code
    pub code: u32,
    /// Error message
    pub message: String,
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

/// When export_service, actor responses will merged by enum type, so if there is two response with same Ok type, the second response will be ignored.
/// So there is no need to create more than one response type for two boolean ok.
#[derive(CandidType)]
pub enum BooleanActorResponse {
    Ok(bool),
    Err(ErrorInfo),
}

impl BooleanActorResponse {
    pub fn new(result: ICNSResult<bool>) -> BooleanActorResponse {
        match result {
            Ok(available) => BooleanActorResponse::Ok(available),
            Err(err) => BooleanActorResponse::Err(err.into()),
        }
    }
}
