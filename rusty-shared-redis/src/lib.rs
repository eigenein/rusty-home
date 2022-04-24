use fred::prelude::*;
use tracing::debug;

/// Replaces an unknown error with [`Result::Ok`].
///
/// This is needed, for example, to safely insert duplicate entries into a stream.
pub fn ignore_unknown_error(error: RedisError) -> Result<(), RedisError> {
    if error.kind() == &RedisErrorKind::Unknown {
        debug!("ignoring error: {:#}", error);
        Ok(())
    } else {
        Err(error)
    }
}
