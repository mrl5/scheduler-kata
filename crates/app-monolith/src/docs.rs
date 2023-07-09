use crate::error::AppError;
use aide::axum::routing::get_with;
use aide::axum::{ApiRouter, IntoApiResponse};
use aide::openapi::OpenApi;
use aide::operation::OperationIo;
use aide::redoc::Redoc;
use axum::response::IntoResponse;
use axum::Extension;
use axum_jsonschema::JsonSchemaRejection;
use axum_macros::FromRequest;
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;

pub fn get_router(oas_path: &str) -> ApiRouter {
    aide::gen::infer_responses(true);

    let router = ApiRouter::new().api_route(
        "/",
        get_with(
            Redoc::new(oas_path)
                .with_title("Scheduler API")
                .axum_handler(),
            |op| op.description("This documentation page."),
        ),
    );

    aide::gen::infer_responses(false);
    router
}

pub async fn serve_oas(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoApiResponse {
    Json(api).into_response()
}

#[derive(FromRequest, OperationIo)]
#[from_request(via(axum_jsonschema::Json), rejection(AppError))]
#[aide(
    input_with = "axum_jsonschema::Json<T>",
    output_with = "axum_jsonschema::Json<T>",
    json_schema
)]
pub struct Json<T>(pub T);

impl<T> IntoResponse for Json<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        axum::Json(self.0).into_response()
    }
}

impl From<JsonSchemaRejection> for AppError {
    fn from(rejection: JsonSchemaRejection) -> Self {
        match rejection {
            JsonSchemaRejection::Json(j) => Self::new(&j.to_string()),
            JsonSchemaRejection::Serde(_) => Self::new("invalid request"),
            JsonSchemaRejection::Schema(s) => {
                Self::new("invalid request").with_details(json!({ "schema_validation": s }))
            }
        }
    }
}
