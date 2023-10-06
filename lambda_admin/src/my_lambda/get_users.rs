use lambda_http::RequestExt;
use lambda_http::{http::StatusCode, lambda_runtime::Context, Request, Response};
use lib_config::config::Config;
use lib_users::services::users::{UserManipulation, UsersService};
use serde_json::json;
//use tracing::instrument;

use super::build_resp;

const PAGESIZE_MAX: u32 = 20;
const PAGESIZE_MIN: u32 = 5;
const PAGENUM_MIN: u32 = 1;

//#[instrument]
pub async fn get_users(
    req: &Request,
    _c: &Context,
    _config: &Config,
    user_service: &UsersService,
) -> Result<Response<String>, Box<dyn std::error::Error>> {
    let page_number;
    //let page_size = PAGESIZE_MIN;
    match req.query_string_parameters().all("pageNumber") {
        None => page_number = PAGENUM_MIN,
        Some(vstr) => {
            let _first = vstr.first();
            match _first {
                None => page_number = PAGENUM_MIN,
                Some(value) => page_number = value.parse::<u32>().unwrap(),
            }
        }
    }
    let page_size;

    match req.query_string_parameters().all("pageSize") {
        None => page_size = PAGESIZE_MIN,
        Some(vstr) => {
            let _first = vstr.first();
            match _first {
                None => page_size = PAGESIZE_MIN,
                Some(value) => page_size = value.parse::<u32>().unwrap(),
            }
        }
    }
    if page_number < PAGENUM_MIN {
        let message = format!("pageNumber is {}, when the minimum value is 1", page_number);
        return build_resp(message.to_string(), StatusCode::BAD_REQUEST);
        //return HttpResponse::BadRequest().body(message);
    }
    if page_size < PAGESIZE_MIN || page_size > PAGESIZE_MAX {
        let message = format!(
            "pageSize is {0}, but it must be between {1} and {2}",
            page_size, PAGENUM_MIN, PAGESIZE_MAX
        );
        //return HttpResponse::BadRequest().body(message);
        return build_resp(message.to_string(), StatusCode::BAD_REQUEST);
    }

    let res = user_service.get_all(page_number, page_size).await;
    match res {
        Err(e) =>
        //HttpResponse::InternalServerError().finish(),
        {
            return build_resp(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR);
        }
        Ok(vec_user) =>
        //HttpResponse::Ok().json(vec_user)
        {
            return build_resp(json!(vec_user).to_string(), StatusCode::OK);
        }
    }
}
