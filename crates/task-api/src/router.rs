use crate::task_v1::{create_task, create_task_docs};
use aide::axum::routing::post_with;
use aide::axum::ApiRouter;

pub fn task_v1_router() -> ApiRouter {
    ApiRouter::new().api_route("/create", post_with(create_task, create_task_docs))
}
