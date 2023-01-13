extern crate zxcvbn;

pub mod errors;
pub mod models;
pub mod repositories;
pub mod services;

use zxcvbn::zxcvbn;
use validator::ValidationError;
pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    
    let op_estimate = zxcvbn(password, &[]);
    match op_estimate {
        Err(e) => { 
            println!("{}",e.to_string());
            return Err(ValidationError::new("")); },
        Ok(value) => {
           if value.score() <=2 {
                return Err(ValidationError::new("Password is too weak. it must contain at least one lower case, one upper case, one number and one symbol of: !@%^#$&*()+=<>?/|.:;'~` and not any sequencial numbers or patterns. Please, improve your password." ));
           } else {
               Ok(())
           }
        }
        
    }
}
