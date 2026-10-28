use error_stuff::CqrsErr;
use my_yrs_lib::{yrs_error::ErrorInfo, YrsError};

pub fn yrs_error_at(err_msg: String, method: &str, file: &str) -> CqrsErr {
    CqrsErr::YrsErrorContainer(YrsError::GenericError {
        info: ErrorInfo {
            error_msg: err_msg,
            file: file.to_string(),
            method: method.to_string(),
        },
    })
}
