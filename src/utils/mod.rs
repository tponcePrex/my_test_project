use crate::datatypes::system_codes::SystemErrorCodes;
use logger::MyError;

pub type CoreError = MyError<SystemErrorCodes>;
pub type CoreResult<T> = Result<T, CoreError>;
pub type MyResult<T> = Result<T, CoreError>;