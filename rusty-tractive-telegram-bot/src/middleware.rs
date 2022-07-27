use poem::error::{MethodNotAllowedError, NotFoundError, ParsePathError, ParseQueryError};
use poem::http::StatusCode;
use poem::{Endpoint, IntoResponse, Middleware, Request, Response, Result};
use tracing::{error, info};

pub struct TracingMiddleware;

impl<E: Endpoint<Output = Response>> Middleware<E> for TracingMiddleware {
    type Output = TracingMiddlewareImpl<E>;

    fn transform(&self, ep: E) -> Self::Output {
        TracingMiddlewareImpl { ep }
    }
}

pub struct TracingMiddlewareImpl<E> {
    ep: E,
}

#[poem::async_trait]
impl<E: Endpoint<Output = Response>> Endpoint for TracingMiddlewareImpl<E> {
    type Output = Response;

    async fn call(&self, request: Request) -> Result<Self::Output> {
        let method = request.method().clone();
        let uri = request.uri().clone();
        match self.ep.call(request).await {
            Err(error) if error.is::<NotFoundError>() => {
                info!(?method, ?uri, "{:#}", error);
                Ok(StatusCode::NOT_FOUND.into_response())
            }
            Err(error) if error.is::<MethodNotAllowedError>() => {
                info!(?method, ?uri, "{:#}", error);
                Ok(StatusCode::METHOD_NOT_ALLOWED.into_response())
            }
            Err(error) => {
                if error.is::<ParseQueryError>() || error.is::<ParsePathError>() {
                    info!(?method, ?uri, "{:#}", error);
                    Ok(StatusCode::BAD_REQUEST.into_response())
                } else {
                    error!(?method, ?uri, "{:#}", error);
                    Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response())
                }
            }
            result => result,
        }
    }
}
