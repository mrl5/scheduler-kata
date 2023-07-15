use crate::task_v1::{
    create_task, create_task_docs, delete_task, delete_task_docs, list_tasks, list_tasks_docs,
    show_task, show_task_docs,
};
use aide::axum::routing::{get_with, post_with};
use aide::axum::ApiRouter;

pub fn task_v1_router() -> ApiRouter {
    ApiRouter::new()
        .api_route(
            "/",
            post_with(create_task, create_task_docs)
                .get_with(show_task, show_task_docs)
                .delete_with(delete_task, delete_task_docs),
        )
        .api_route("/list", get_with(list_tasks, list_tasks_docs))
}
