use utoipa::OpenApi;
use crate::models::{
    AsyncTaskResponse, ConversionResponse, ErrorResponse, HealthResponse, TaskStatus,
    TaskStatusResponse,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        convert_pdf,
        get_task_status,
        get_openapi_json
    ),
    components(
        schemas(
            HealthResponse,
            ConversionResponse,
            AsyncTaskResponse,
            TaskStatus,
            TaskStatusResponse,
            ErrorResponse
        )
    ),
    tags(
        (name = "PaperFlux HTTP API", description = "High-Performance Universal PDF to Markdown Intelligence Microservice")
    ),
    info(
        title = "PaperFlux PDF Intelligence Microservice",
        version = "0.1.0",
        description = "REST API for converting complex multilingual and Arabic-first PDF documents to Markdown with async task handling."
    )
)]
pub struct ApiDoc;

#[utoipa::path(
    get,
    path = "/health",
    tag = "System",
    responses(
        (status = 200, description = "Service health check", body = HealthResponse)
    )
)]
pub fn health_check() {}

#[utoipa::path(
    post,
    path = "/convert",
    tag = "Conversion",
    request_body(
        content = Vec<u8>,
        description = "Raw PDF binary payload",
        content_type = "application/pdf"
    ),
    params(
        ("async" = Option<bool>, Query, description = "Enable asynchronous processing (returns 202 Accepted with task_id)"),
        ("dialect" = Option<String>, Query, description = "Markdown dialect (gfm, commonmark, extended)"),
        ("detect_tables" = Option<bool>, Query, description = "Enable table detection and extraction")
    ),
    responses(
        (status = 200, description = "Synchronous conversion result", body = ConversionResponse),
        (status = 202, description = "Asynchronous task accepted", body = AsyncTaskResponse),
        (status = 400, description = "Bad request or invalid PDF", body = ErrorResponse)
    )
)]
pub fn convert_pdf() {}

#[utoipa::path(
    get,
    path = "/status/{task_id}",
    tag = "Conversion",
    params(
        ("task_id" = String, Path, description = "Unique conversion task identifier")
    ),
    responses(
        (status = 200, description = "Task status and result if completed", body = TaskStatusResponse),
        (status = 404, description = "Task not found", body = ErrorResponse)
    )
)]
pub fn get_task_status() {}

#[utoipa::path(
    get,
    path = "/api-docs/openapi.json",
    tag = "Documentation",
    responses(
        (status = 200, description = "OpenAPI 3.0 JSON specification")
    )
)]
pub fn get_openapi_json() {}
