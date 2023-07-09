use crate::error::AppError;
use crate::extractor::Json;
use aide::axum::IntoApiResponse;
use aide::openapi::{self, OpenApi};
use aide::transform::TransformOpenApi;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Extension;
use std::sync::Arc;

pub fn init_openapi() -> OpenApi {
    aide::gen::on_error(|error| {
        tracing::error!("Aide generation error: {error}");
    });
    aide::gen::extract_schemas(true);
    aide::gen::infer_responses(true);
    aide::gen::inferred_empty_response_status(204);

    aide::gen::in_context(|ctx| ctx.schema = schemars::gen::SchemaSettings::openapi3().into());

    OpenApi {
        info: openapi::Info {
            title: "Scheduler API".to_owned(),
            version: "0.1.0".to_owned(),
            description: Some("Scheduler Kata".to_string()),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn api_docs(api: TransformOpenApi) -> TransformOpenApi {
    api.title("Scheduler API")
        .description("Scheduler Kata")
        .default_response_with::<Json<AppError>, _>(|res| {
            res.example(AppError {
                error: "some error happened".to_string(),
                error_details: None,
                // This is not visible.
                status: StatusCode::IM_A_TEAPOT,
            })
        })
}

pub async fn serve_oas(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoApiResponse {
    Json(api).into_response()
}
