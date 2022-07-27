use poem::http::header::HeaderName;
use poem::http::HeaderValue;
use poem::web::headers::{Error, Header};
use secstr::SecUtf8;

#[derive(Clone)]
pub struct SecretToken(pub SecUtf8);

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
        values
            .next()
            .ok_or_else(Error::invalid)?
            .to_str()
            .map_err(|_| Error::invalid())
            .map(SecUtf8::from)
            .map(Self)
    }

    fn encode<E: Extend<HeaderValue>>(&self, _values: &mut E) {
        unimplemented!()
    }
}

impl SecretToken {
    pub fn is_valid(&self, secret_token: &SecUtf8) -> bool {
        &self.0 == secret_token
    }
}
