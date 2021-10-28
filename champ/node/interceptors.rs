use pog_jwt::verify;
use tonic::{metadata::MetadataValue, Request, Status};

#[derive(Clone)]
pub struct Interceptor {
    pub public_key: String,
}

pub fn interceptor_auth(req: Request<()>, public_key: &str) -> Result<Request<()>, Status> {
    if let Some(t) = req.metadata().get("authorization") {
        let token = MetadataValue::to_str(t).map_err(|_| Status::unauthenticated("token could not be parsed"))?;
        let _ = verify(token, public_key.as_bytes()).map_err(|_| Status::unauthenticated("No auth token provided"))?;

        //req.extensions_mut().insert(MyExtension {
        //    some_piece_of_data: "foo".to_string(),
        //});

        return Ok(req);
    }
    Err(Status::unauthenticated("No auth token provided"))
}
