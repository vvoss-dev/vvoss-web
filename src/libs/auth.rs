use actix_web::{dev::ServiceRequest, Error};
use actix_web_httpauth::extractors::basic::BasicAuth;
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web_httpauth::headers::www_authenticate::basic::Basic;

use super::config::Config;

pub async fn validator(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let config = req.app_data::<actix_web::web::Data<Config>>()
        .map(|c| c.get_ref().clone())
        .unwrap();

    if !config.auth.enabled {
        return Ok(req);
    }

    match (credentials.user_id(), credentials.password()) {
        (user, Some(pass)) if user == config.auth.username && pass == config.auth.password => {
            Ok(req)
        }
        _ => {
            let challenge = Basic::default();
            Err((AuthenticationError::new(challenge).into(), req))
        }
    }
}