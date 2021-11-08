use anyhow::Result;
use tonic::{Code, Status};

use crate::auth::interceptors::UserMetadata;

//Current Permissions:
// "admin.read"             -> Read-access to the NodeAdmin Service
// "admin.write"            -> Write-access to the NodeAdmin Service
// "admin.logs"             -> Read-access to all logs
// "wallet.create"          -> Create a wallet on a node
// "wallet.{id}.sign"       -> Write.access to a wallet with {id}
// "wallet.{id}.manage"     -> Edit.access to a wallet with {id}
// "wallet.{id}.unlock"     -> Write.access to a wallet with {id}
// "superadmin"             -> Access to everything

pub fn verify_perms<T>(request: &tonic::Request<T>, needs: &str) -> Result<(), tonic::Status> {
    let permissions = &request
        .extensions()
        .get::<UserMetadata>()
        .ok_or_else(|| Status::new(Code::Unauthenticated, "Metadata could not be found"))?
        .permissions;

    if permissions.contains(&"superadmin".to_string()) {
        return Ok(());
    }

    if permissions.contains(&needs.to_string()) {
        return Ok(());
    }

    Err(Status::new(Code::Unauthenticated, "No permissions"))
}
