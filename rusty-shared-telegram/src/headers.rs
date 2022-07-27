use poem::http::header::HeaderName;
use poem::http::HeaderValue;
use poem::web::headers::{Error, Header};
use secstr::SecVec;

#[derive(Clone)]
pub struct SecretToken(pub SecVec<u8>);

static X_SECRET_TOKEN_NAME: HeaderName = HeaderName::from_static("x-telegram-bot-api-secret-token");

impl Header for SecretToken {
    fn name() -> &'static HeaderName {
        &X_SECRET_TOKEN_NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values.next().ok_or_else(Error::invalid)?;
        if value.is_empty() {
            return Err(Error::invalid());
        }
        hex::decode(value.as_bytes())
            .map(SecVec::new)
            .map(Self)
            .map_err(|_| Error::invalid())
    }

    fn encode<E: Extend<HeaderValue>>(&self, _values: &mut E) {
        unimplemented!()
    }
}
