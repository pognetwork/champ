use pog_jwt::verify;
use tonic::{metadata::MetadataValue, Request, Status};

use crate::state::ChampState;

async fn check_auth(req: Request<()>, state: ChampState) -> Result<Request<()>, Status> {
    let metadata_token = MetadataValue::from_str("Bearer some-secret-token").unwrap();
    let token =
        metadata_token.to_str().map_err(|_| Status::new(tonic::Code::Internal, "token could not be retrieved"))?;
    let public_key = &state.config.read().await.admin.jwt_public_key;
    if !verify(token, public_key.as_bytes()).is_ok() {
        return Err(Status::unauthenticated("No valid auth token"));
    }

    match req.metadata().get("authorization") {
        Some(t) if token == t => Ok(req),
        _ => Err(Status::unauthenticated("No valid auth token")),
    }
}
