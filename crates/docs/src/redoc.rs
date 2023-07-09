use aide::axum::routing::get_with;
use aide::axum::ApiRouter;
use aide::redoc::Redoc;

pub fn get_router(oas_path: &str) -> ApiRouter {
    aide::gen::infer_responses(true);

    let router = ApiRouter::new().api_route(
        "/",
        get_with(
            Redoc::new(oas_path)
                .with_title("Scheduler API")
                .axum_handler(),
            |op| op.description("This documentation page"),
        ),
    );

    aide::gen::infer_responses(false);
    router
}
