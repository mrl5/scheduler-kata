use crate::task_v1::{
    create_task, create_task_docs, delete_task, delete_task_docs, show_task, show_task_docs,
};
use aide::axum::routing::post_with;
use aide::axum::ApiRouter;

pub fn task_v1_router() -> ApiRouter {
    ApiRouter::new().api_route(
        "/",
        post_with(create_task, create_task_docs)
            .get_with(show_task, show_task_docs)
            .delete_with(delete_task, delete_task_docs),
    )
}
