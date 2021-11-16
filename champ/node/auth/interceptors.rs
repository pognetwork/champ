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
        let claims = verify(token, public_key.as_bytes())
            .map_err(|_| Status::unauthenticated("Invalid auth token provided"))?;

        let (_, user) = users
            .iter()
            .find(|(_, acc)| acc.user_id == claims.sub)
            .ok_or_else(|| Status::unauthenticated("user not found"))?;
        tracing::trace!("user authenticated; permissions={:?}", user.permissions.clone());

        req.extensions_mut().insert(UserMetadata {
            permissions: user.permissions.clone(),
        });

        return Ok(req);
    }

    tracing::debug!("got unauthenticated request to authenticated service");
    Err(Status::unauthenticated("No auth token provided"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crypto::{
        id,
        password::{generate_salt, hash},
        signatures::ecdsa::{generate_key_pair, PEMKeyPair},
    };

    fn mock() -> (Request<()>, PEMKeyPair, UserAccount, BTreeMap<String, UserAccount>) {
        let mut users = BTreeMap::new();
        let user = UserAccount {
            permissions: vec![],
            user_id: id::generate().expect("should generate user_id"),
            password_hash: hash(b"test", &generate_salt().expect("should generate salt"))
                .expect("should hash password"),
        };
        users.insert("test".to_string(), user.clone());
        let req = Request::new(());
        let key_pair = generate_key_pair().expect("should generate keypair");
        (req, key_pair, user, users)
    }

    #[test]
    fn unauthenticated() {
        let (req, key_pair, _, users) = mock();
        interceptor_auth(req, &key_pair.public_key, &users).expect_err("should not allow missing token");
    }

    #[test]
    fn jwt_tokens() {
        let (mut req, key_pair, user, users) = mock();

        let jwt =
            &pog_jwt::create(&user.user_id, "test", 10, key_pair.private_key.as_bytes()).expect("should create jwt");
        req.metadata_mut()
            .append("authorization", MetadataValue::from_str(jwt).expect("matadata value should be created"));

        verify(jwt, key_pair.public_key.as_bytes()).expect("valid token");
        interceptor_auth(req, &key_pair.public_key, &users).expect("should allow valid token");
    }

    #[test]
    fn should_err_on_invalid_token() {
        let (mut req, key_pair, _, users) = mock();
        req.metadata_mut()
            .append("authorization", MetadataValue::from_str("test").expect("matadata value should be created"));
        interceptor_auth(req, &key_pair.public_key, &users).expect_err("should disallow invalid token");
    }

    #[test]
    fn should_err_on_invalid_user() {
        let (mut req, key_pair, _, users) = mock();
        req.metadata_mut().append(
            "authorization",
            MetadataValue::from_str(
                &pog_jwt::create(&"invalid_user_id", "invalid_user", 10, key_pair.private_key.as_bytes())
                    .expect("shoud create jwt"),
            )
            .expect("matadata value should be created"),
        );
        interceptor_auth(req, &key_pair.public_key, &users).expect_err("should disallow nonexisting user");
    }
}
