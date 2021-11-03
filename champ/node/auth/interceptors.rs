use std::collections::BTreeMap;

use pog_jwt::verify;
use tonic::{metadata::MetadataValue, Request, Status};

use crate::config::UserAccount;

#[derive(Clone)]
pub struct Interceptor {
    pub public_key: String,
}

pub struct UserMetadata {
    pub permissions: Vec<String>,
}

pub fn interceptor_auth(
    mut req: Request<()>,
    public_key: &str,
    users: &BTreeMap<String, UserAccount>,
) -> Result<Request<()>, Status> {
    if let Some(t) = req.metadata().get("authorization") {
        let token = MetadataValue::to_str(t).map_err(|_| Status::unauthenticated("token could not be parsed"))?;
        let claims =
            verify(token, public_key.as_bytes()).map_err(|_| Status::unauthenticated("No auth token provided"))?;

        let user = users.get(&claims.sub).ok_or_else(|| Status::unauthenticated("user not found"))?;

        req.extensions_mut().insert(UserMetadata {
            permissions: user.permissions.clone(),
        });

        return Ok(req);
    }
    Err(Status::unauthenticated("No auth token provided"))
}
