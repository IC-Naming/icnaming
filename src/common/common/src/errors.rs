use candid::{CandidType, Deserialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use thiserror::Error;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, CandidType, Deserialize, Error)]
pub enum NamingError {
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
    #[error("some operations are processing, please try again later")]
    Conflict,
    #[error("insufficient quota")]
    InsufficientQuota,
    #[error("it is not allowed to renew the name more than {years:?} years")]
    RenewalYearsError { years: u32 },
    #[error("price changed, please refresh and try again")]
    InvalidApproveAmount,
}

impl NamingError {
    pub(crate) fn code(&self) -> u32 {
        match self {
            NamingError::Unknown => 1,
            NamingError::RemoteError(_) => 2,
            NamingError::InvalidCanisterName => 3,
            NamingError::InvalidOwner => 4,
            NamingError::OwnerOnly => 5,
            NamingError::InvalidName { .. } => 6,
            NamingError::NameUnavailable { .. } => 7,
            NamingError::PermissionDenied => 8,
            NamingError::RegistrationHasBeenTaken => 9,
            NamingError::RegistrationNotFound => 10,
            NamingError::TopNameAlreadyExists => 11,
            NamingError::RegistryNotFoundError { .. } => 12,
            NamingError::ResolverNotFoundError { .. } => 13,
            NamingError::OperatorShouldNotBeTheSameToOwner => 14,
            NamingError::YearsRangeError { .. } => 15,
            NamingError::InvalidResolverKey { .. } => 16,
            NamingError::ValueMaxLengthError { .. } => 17,
            NamingError::ValueShouldBeInRangeError { .. } => 18,
            NamingError::TooManyFavorites { .. } => 19,
            NamingError::Unauthorized => 20,
            NamingError::InvalidQuotaOrderDetails => 21,
            NamingError::PendingOrder => 22,
            NamingError::OrderNotFound => 23,
            NamingError::RefundFailed => 24,
            NamingError::OperatorCountExceeded => 25,
            NamingError::CanisterCallError { .. } => 26,
            NamingError::InvalidResolverValueFormat { .. } => 27,
            NamingError::Conflict => 28,
            NamingError::InsufficientQuota => 29,
            NamingError::RenewalYearsError { .. } => 30,
            NamingError::InvalidApproveAmount => 31,
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

impl Display for ErrorInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.code, self.message)
    }
}

pub fn get_error_code(error: NamingError) -> ErrorInfo {
    ErrorInfo {
        code: error.code(),
        message: error.to_string(),
    }
}

pub type ServiceResult<T> = anyhow::Result<T, NamingError>;

/// A helper function to convert anyhow::Result<T, ICNSError> to ICNSResult<T>
pub type ActorResult<T> = Result<T, ErrorInfo>;

impl From<NamingError> for ErrorInfo {
    fn from(error: NamingError) -> Self {
        get_error_code(error)
    }
}

impl From<ErrorInfo> for NamingError {
    fn from(error: ErrorInfo) -> Self {
        NamingError::RemoteError(error)
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
    pub fn new(result: ServiceResult<bool>) -> BooleanActorResponse {
        match result {
            Ok(available) => BooleanActorResponse::Ok(available),
            Err(err) => BooleanActorResponse::Err(err.into()),
        }
    }
}
