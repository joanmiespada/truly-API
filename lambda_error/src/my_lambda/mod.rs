use lib_config::result::ResultE;
use lib_licenses::services::assets::{AssetService, AssetManipulation };
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug)]
pub struct ErrorParsingError(pub String);

impl std::error::Error for ErrorParsingError {}

impl std::fmt::Display for ErrorParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error parsing error: {}", self.0)
    }
}


pub async fn save_error(data: Value, asset_service: &AssetService) -> ResultE<()> {

    let asset_value = match data.get("asset_id").and_then(Value::as_str)
    {
        Some(value)=>value, 
        None => return Err(ErrorParsingError("asset_id field has no correct value".to_string()).into())
    };
    
    let exception_value = match data.get("exception").and_then(Value::as_str)
    {
        Some(value)=>String::from(value),
        None => return Err(ErrorParsingError("exception field has no correct value".to_string()).into())
    };
    
    let stage_value = match data.get("stage").and_then(Value::as_str)
    {
        Some(value)=> String::from(value), 
        None => return Err(ErrorParsingError("stage field has no correct value".to_string()).into())
    };


    let asset_id =  Uuid::parse_str(asset_value)?;

    let mut asset = asset_service.get_by_id(&asset_id).await?;

    asset.set_hash_process_status(&Some(lib_licenses::models::asset::HashProcessStatus::Error));
    asset.set_hash_process_error_stage(&Some(stage_value));
    asset.set_hash_process_error_message(&Some(exception_value));

    asset_service.update_full(&asset).await?;

    Ok(())
}
