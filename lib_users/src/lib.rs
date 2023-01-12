//extern crate derive_more;
//extern crate rustc_serialize;
extern crate zxcvbn;

pub mod errors;
pub mod models;
pub mod repositories;
pub mod services;

use zxcvbn::zxcvbn;
use regex::Regex;
use validator::ValidationError;
//"^(?=.*[A-Z])(?=.*[!@%^#$&*()+=<>?/|:;'~`])(?=.*[0-9])(?=.*[a-z])$"
pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    /* 
    let re = Regex::new(r"^(?=.*[A-Z])$");
    match re {
        Err(e) => { 
            println!("{}",e.to_string());
            return Err(ValidationError::new("")); },
        Ok(value) => {
            if !value.is_match(password) {
                // the value of the username will automatically be added later
                return Err(ValidationError::new("password must contain at least one lower case, one upper case, one number and one symbol of: !@%^#$&*()+=<>?/|:;'~`"));
            }
            Ok(())
        }
    }*/
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
