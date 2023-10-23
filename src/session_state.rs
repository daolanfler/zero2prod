use std::future::{ready, Ready};

use actix_session::{Session, SessionExt, SessionGetError, SessionInsertError};
use actix_web::{dev::Payload, FromRequest, HttpRequest};
use uuid::Uuid;

// extension trait pattern
pub struct TypedSession(Session);

impl TypedSession {
    const USER_ID_KEY: &'static str = "user_id";

    pub fn renew(&self) {
        self.0.renew();
    }

    pub fn insert_user_id(&self, user_id: Uuid) -> Result<(), SessionInsertError> {
        self.0.insert(Self::USER_ID_KEY, user_id)
    }

    pub fn get_user_id(&self) -> Result<Option<Uuid>, SessionGetError> {
        self.0.get(Self::USER_ID_KEY)
    }
}

impl FromRequest for TypedSession {
    // This is a complicated way of saying
    // "We return the same error returned by the implementation of `FromRequst`
    // for `Session`"
    type Error = <Session as FromRequest>::Error;
    // Rust does not yet support the `async` syntax in traits.
    // FromRequest expects a `Future` as return type to allow for extractors
    // that need to perform asynchronous oeprations (e.g. a HTTP call)
    // We do not have a `Future`, because we don't perform any I/O,
    // so we wrap `TypedSession` into `Ready` to convert it into a `Future` that
    // resolves to the wrapped value the first time it's polled by the executor.
    type Future = Ready<Result<TypedSession, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(Ok(TypedSession(req.get_session())))
    }
}