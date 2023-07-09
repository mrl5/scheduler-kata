use crate::health_check::{run_healthcheck, run_healthcheck_docs};
use aide::axum::routing::get_with;
use aide::axum::ApiRouter;

pub fn get_router() -> ApiRouter {
    ApiRouter::new().api_route("/health", get_with(run_healthcheck, run_healthcheck_docs))
}
