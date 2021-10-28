use pog_jwt::verify;
//use serde::{Deserialize, Serialize};
use tonic::{metadata::MetadataValue, Request, Status};

#[derive(Clone)]
pub struct Interceptor {
    pub public_key: String,
}

//#[derive(Serialize, Deserialize, PartialEq)]
//pub enum Permissions {
//    #[serde(rename = "node_admin")]
//    NodeAdmin,
//}

pub struct UserMetadata {
    pub permissions: Vec<String>,
}

pub fn interceptor_auth(mut req: Request<()>, public_key: &str) -> Result<Request<()>, Status> {
    if let Some(t) = req.metadata().get("authorization") {
        let token = MetadataValue::to_str(t).map_err(|_| Status::unauthenticated("token could not be parsed"))?;
        let claims =
            verify(token, public_key.as_bytes()).map_err(|_| Status::unauthenticated("No auth token provided"))?;

        req.extensions_mut().insert(UserMetadata {
            permissions: claims.permissions,
        });

        return Ok(req);
    }
    Err(Status::unauthenticated("No auth token provided"))
}
