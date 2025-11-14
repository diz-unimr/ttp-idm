use crate::error::ApiError;
pub(crate) use crate::model::IdRequest;
use crate::model::IdResponse;
use crate::server::ApiContext;
use anyhow::anyhow;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{debug_handler, Json, Router};
use fhir_model::r4b::resources::{ParametersParameter, Person};
use reqwest::StatusCode;

pub(crate) fn router() -> Router<ApiContext> {
    Router::new().route("/api/pseudonyms", post(create))
}

#[debug_handler]
pub(crate) async fn create(
    State(ctx): State<ApiContext>,
    Json(payload): Json<IdRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // 1. create mpi in epix or return on conflict
    let res = ctx.client.add_person(payload.clone()).await?;

    // todo check matchStatus
    // return 409 conflict on match?
    let parts: Vec<ParametersParameter> = res
        .parameter
        .iter()
        .flatten()
        .filter_map(|p| {
            if p.name == "matchResult" {
                Some(p.part.iter().flatten())
            } else {
                None
            }
        })
        .flatten()
        .cloned()
        .collect();

    // mpi person resource
    let person = parts
        .into_iter()
        .filter_map(|part| {
            if part.name == "mpiPerson" {
                Some(part.resource.and_then(|p| Person::try_from(p).ok()))
            } else {
                None
            }
        })
        .flatten()
        .next()
        .ok_or(anyhow!("Failed to parse mpiPerson from E-PIX response"))?;

    let mpi: String = person
        .identifier
        .iter()
        .flatten()
        .filter_map(|i| {
            if i.system == Some("https://ths-greifswald.de/fhir/epix/identifier/MPI".to_string()) {
                i.value.clone()
            } else {
                None
            }
        })
        .next()
        .ok_or(anyhow!(
            "Failed to parse MPI identifier from E-PIX response"
        ))?;

    // 2. create pseudonyms
    let (patient_id, lab) = ctx.client.pseudonymize(mpi.clone(), payload).await?;

    Ok((StatusCode::OK, Json(IdResponse { patient_id, lab })))
}
