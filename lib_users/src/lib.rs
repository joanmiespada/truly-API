extern crate zxcvbn;

pub mod errors;
pub mod models;
pub mod repositories;
pub mod services;

use validator::ValidationError;
use zxcvbn::zxcvbn;
pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    let op_estimate = zxcvbn(password, &[]);
    match op_estimate {
        Err(_) => {
            return Err(ValidationError::new("Error processing password quality"));
        }
        Ok(value) => {
            if value.score() < 3 {
                return Err(ValidationError::new("Password is too weak. it must contain at least one lower case, one upper case, one number and one symbol of: !@%^#$&*()+=<>?/|.:;'~` and not any sequencial numbers or patterns. Please, improve your password." ));
            } else {
                Ok(())
            }
        }
    }
}
