use crate::error::ApiError;
pub(crate) use crate::model::IdRequest;
use crate::model::{IdMatch, IdResponse, MatchStatus, PromptResponse};
use crate::server::ApiContext;
use crate::ttp::client::TtpClient;
use anyhow::anyhow;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{debug_handler, Json, Router};
use fhir_model::r4b::resources::{
    Parameters, ParametersParameter, ParametersParameterValue, Patient, Person,
};
use reqwest::StatusCode;
use std::sync::Arc;

pub(crate) fn router() -> Router<Arc<ApiContext>> {
    Router::new()
        .route("/api/pseudonyms/{trial}/{psn}", get(read))
        .route("/api/pseudonyms", post(create))
}

#[debug_handler]
#[utoipa::path(post, path = "/api/pseudonyms", responses((status = OK, body = IdResponse)))]
pub(crate) async fn create(
    State(ctx): State<Arc<ApiContext>>,
    Json(payload): Json<IdRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // get/create mpi in epix
    let res = ctx.client.add_person(payload.clone()).await?;

    // parse response
    match match_status(&res)? {
        MatchStatus::MatchError => Err(anyhow!("E-PIX addPerson failed with MatchError"))?,
        MatchStatus::MultipleMatch => {
            log::error!("MultipleMatch");
            todo!("handle multiple matches")
        }
        MatchStatus::ExternalMatch => {
            log::error!("ExternalMatch");
            todo!("handle these")
        }
        MatchStatus::PerfectMatchWithUpdate => {
            log::error!("PerfectMatchWithUpdate");
            todo!("handle these")
        }
        MatchStatus::Match => {
            log::error!("Match");
            todo!("handle these")
        }
        MatchStatus::PossibleMatch => {
            // get possible matches
            let mut mpi = parse_mpi(&res)?;
            let possible_matches = ctx.client.possible_matches_for_person(mpi.clone()).await?;

            // newly created identity_id
            let identity_id = parse_identity_id(&res)?;

            // resolve match
            if let Some(link) = &payload.link {
                if link.merge {
                    // delete newly created entity
                    ctx.client.delete_identity(identity_id.parse()?).await?;

                    // matched mpi
                    mpi = possible_matches
                        .into_iter()
                        .find_map(|p| {
                            if p.matching_identity.identity.identity_id == link.id {
                                return Some(p.matching_identity.mpi_id.value);
                            }
                            None
                        })
                        .ok_or(anyhow!("Target mpi not found"))?;
                } else {
                    // dont merge: remove possible matches
                    for p in possible_matches {
                        ctx.client.split_identities(p.link_id).await?;
                    }
                }

                // create pseudonyms
                let (participant, lab) = ctx.client.pseudonymize(mpi, payload).await?;
                Ok((StatusCode::OK, Json(IdResponse { participant, lab })).into_response())
            } else {
                // or prompt for matches:

                // delete newly created entity
                ctx.client.delete_identity(identity_id.parse()?).await?;

                // return conflicting match
                let matches = possible_matches
                    .into_iter()
                    .map(|m| m.matching_identity.identity.into())
                    .collect::<Vec<IdMatch>>();

                let prompt_response = PromptResponse { matches };

                Ok((StatusCode::CONFLICT, Json(prompt_response)).into_response())
            }
        }
        MatchStatus::NoMatch | MatchStatus::PerfectMatch => {
            // parse mpi from response
            let mpi = parse_mpi(&res)?;

            // create pseudonyms
            let (participant, lab) = ctx.client.pseudonymize(mpi.clone(), payload).await?;

            Ok((StatusCode::OK, Json(IdResponse { participant, lab })).into_response())
        }
    }
}

#[debug_handler]
#[utoipa::path(get, path = "/api/pseudonyms", responses((status = OK, body = IdResponse)))]
pub(crate) async fn read(
    State(ctx): State<Arc<ApiContext>>,
    Path((trial, psn)): Path<(String, String)>,
) -> Result<impl IntoResponse, ApiError> {
    // get mpi
    let mpi = ctx.client.identify(trial.clone(), psn.clone()).await?;

    // get domains
    let domains = ctx.client.get_secondary_domains(trial.clone()).await?;

    // get pseudonyms
    let client: Arc<TtpClient> = Arc::new(ctx.client.clone());
    let lab = client.get_pseudonyms(domains, mpi).await?;

    Ok((
        StatusCode::OK,
        Json(IdResponse {
            participant: psn,
            lab,
        }),
    ))
}

fn match_status(params: &Parameters) -> anyhow::Result<MatchStatus> {
    let match_code: anyhow::Result<&str> = params
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
        .find_map(|p| {
            if p.name == "matchStatus" {
                return match &p.value {
                    Some(ParametersParameterValue::Coding(c)) => {
                        match c.code.as_deref() {
                            Some(code) => Some(Ok(code)),
                            None => Some(Err(anyhow!(
                                "Failed to parse matchStatus of E-PIX response. Missing code"
                            ))),
                        }
                    }
                    _ => Some(Err(anyhow!(
                        "Failed to parse matchStatus of E-PIX response. Value is not a Coding: {:#?}",
                        p.value
                    ))),
                };
            }
            None
        }).ok_or(anyhow!(
        "Failed to parse matchStatus of E-PIX response. No 'matchStatus' Parameter found"
    ))?;

    match_code.map(MatchStatus::try_from)?
}

fn parse_mpi(params: &Parameters) -> Result<String, anyhow::Error> {
    // mpi person resource
    let person = match_result(params)
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

    person
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
        ))
}

fn match_result(params: &Parameters) -> Vec<ParametersParameter> {
    params
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
        .collect()
}

fn parse_identity_id(params: &Parameters) -> Result<String, anyhow::Error> {
    // mpi person resource
    match_result(params)
        .into_iter()
        .filter_map(|part| {
            if part.name == "identity" {
                Some(
                    part.resource
                        .and_then(|p| Patient::try_from(p).ok().and_then(|p| p.id.clone())),
                )
            } else {
                None
            }
        })
        .flatten()
        .next()
        .ok_or(anyhow!("Failed to parse person_id from E-PIX response"))
}
