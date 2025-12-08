use crate::error::ApiError;
pub(crate) use crate::model::IdRequest;
use crate::model::{IdResponse, Idat, MatchStatus, PromptResponse};
use crate::server::ApiContext;
use crate::ttp::client::TtpClient;
use anyhow::anyhow;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{debug_handler, Json, Router};
use fhir_model::r4b::resources::{
    Parameters, ParametersParameter, ParametersParameterValue, Person,
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
    // 1. check request for merge 'flag'
    if let Some(link) = &payload.link {
        if link.merge {
            // merge duplicate
            ctx.client.merge_identities(link.id).await?;
        } else {
            // keep separate identity
            ctx.client.split_identities(link.id).await?;
        }
    }

    // 2. get/create mpi in epix or return on conflict
    let res = ctx.client.add_person(payload.clone()).await?;

    // 3. parse response
    match match_status(&res)? {
        MatchStatus::MatchError => Err(anyhow!("E-PIX addPerson failed with MatchError"))?,
        MatchStatus::MultipleMatch => {
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
            // get possible match via getPossibleMatchesForPerson
            let mpi = parse_mpi(res)?;
            let match_response = ctx.client.possible_matches_for_person(mpi).await?;
            // which one if multiple?
            let match_response = match_response.first().ok_or(anyhow!("No match"))?;

            // todo parse response
            let link_id = match_response.link_id;
            let idat: Idat = match_response.matching_identity.identity.clone().into();
            let prompt_response = PromptResponse { idat, link_id };

            let resp = (StatusCode::CONFLICT, Json(prompt_response)).into_response();
            Ok(resp)
        }
        MatchStatus::NoMatch | MatchStatus::PerfectMatch => {
            // 4. parse mpi from response
            let mpi = parse_mpi(res)?;

            // 5. create pseudonyms
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
    let mpi = ctx.client.identify(trial.clone(), psn).await?;

    // get domains
    let domains = ctx.client.get_secondary_domains(trial.clone()).await?;

    // get pseudonyms
    let client: Arc<TtpClient> = Arc::new(ctx.client.clone());
    let lab = client.get_pseudonyms(domains, mpi.clone()).await?;

    Ok((
        StatusCode::OK,
        Json(IdResponse {
            participant: mpi,
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

fn parse_mpi(params: Parameters) -> Result<String, anyhow::Error> {
    let parts: Vec<ParametersParameter> = params
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
