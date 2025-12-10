use crate::ttp::epix::model::MpiIdentity;
use anyhow::anyhow;
use chrono::{Datelike, NaiveDate};
use fhir_model::r4b::codes::NameUse;
use fhir_model::r4b::resources::{Patient, Resource};
use fhir_model::r4b::types::{Address, Extension, ExtensionValue, HumanName, Meta};
use fhir_model::Date;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use time::error::ComponentRange;

pub(crate) enum MatchStatus {
    NoMatch,
    PerfectMatch,
    ExternalMatch,
    Match,
    MatchError,
    MultipleMatch,
    PerfectMatchWithUpdate,
    PossibleMatch,
}

impl TryFrom<&str> for MatchStatus {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "NO_MATCH" => Ok(MatchStatus::NoMatch),
            "PERFECT_MATCH" => Ok(MatchStatus::PerfectMatch),
            "EXTERNAL_MATCH" => Ok(MatchStatus::ExternalMatch),
            "MATCH" => Ok(MatchStatus::Match),
            "MATCH_ERROR" => Ok(MatchStatus::MatchError),
            "MULTIPLE_MATCH" => Ok(MatchStatus::MultipleMatch),
            "PERFECT_MATCH_WITH_UPDATE" => Ok(MatchStatus::PerfectMatchWithUpdate),
            "POSSIBLE_MATCH" => Ok(MatchStatus::PossibleMatch),
            other => Err(anyhow!(
                "Failed to parse MatchStatus. Unknown code: {other}"
            )),
        }
    }
}

#[derive(utoipa::ToSchema, Serialize)]
pub(crate) struct IdResponse {
    pub(crate) participant: String,
    pub(crate) lab: HashMap<String, Vec<String>>,
}

#[derive(utoipa::ToSchema, Deserialize, Serialize, Debug, PartialEq, Clone)]
pub(crate) struct Idat {
    pub(crate) first_name: String,
    pub(crate) last_name: String,
    pub(crate) birth_date: NaiveDate,
    pub(crate) birth_place: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) birth_name: Option<String>,
    pub(crate) postal_code: String,
    pub(crate) city: String,
}

#[derive(utoipa::ToSchema, Deserialize, Serialize)]
pub(crate) struct PromptResponse {
    pub(crate) matches: Vec<IdMatch>,
}

#[derive(utoipa::ToSchema, Deserialize, Serialize)]
pub(crate) struct IdMatch {
    pub(crate) idat: Idat,
    pub(crate) link_id: u32,
}

#[derive(utoipa::ToSchema, Deserialize, Clone)]
pub(crate) struct Link {
    pub(crate) id: u32,
    pub(crate) merge: bool,
}

#[derive(utoipa::ToSchema, Deserialize, Clone)]
pub(crate) struct IdRequest {
    pub(crate) idat: Idat,
    pub(crate) trial: String,
    pub(crate) lab: HashMap<String, u32>,
    pub(crate) link: Option<Link>,
}

impl TryInto<Patient> for IdRequest {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Patient, Self::Error> {
        // first name, last name
        let mut names = vec![Some(
            HumanName::builder()
                .given(
                    self.idat
                        .first_name
                        .split(' ')
                        .map(|n| Some(n.to_string()))
                        .collect::<Vec<_>>(),
                )
                .family(self.idat.last_name)
                .build()?,
        )];

        // (optional) birth name
        if let Some(name) = self.idat.birth_name {
            names.push(Some(
                HumanName::builder()
                    .r#use(NameUse::Maiden)
                    .family(name)
                    .build()?,
            ));
        }

        let builder = Patient::builder()
            .meta(
                Meta::builder()
                    .profile(vec![Some(
                        "https://ths-greifswald.de/fhir/StructureDefinition/epix/Patient"
                            .to_string(),
                    )])
                    .build()?,
            )
            .name(names)
            .birth_date(Date::Date(parse_date(self.idat.birth_date)?))
            .address(vec![Some(
                Address::builder()
                    .postal_code(self.idat.postal_code)
                    .city(self.idat.city)
                    .build()?,
            )])
            .extension(vec![
                Extension::builder()
                    .url("http://hl7.org/fhir/StructureDefinition/patient-birthPlace".to_string())
                    .value(ExtensionValue::Address(
                        Address::builder().city(self.idat.birth_place).build()?,
                    ))
                    .build()?,
            ]);

        Ok(builder.build()?)
    }
}

impl From<MpiIdentity> for Idat {
    fn from(value: MpiIdentity) -> Self {
        Idat {
            first_name: value.first_name,
            last_name: value.last_name,
            birth_date: value.birth_date,
            birth_place: value.birth_place,
            birth_name: value.mothers_maiden_name,
            postal_code: value.contacts.zip_code,
            city: value.contacts.city,
        }
    }
}

impl From<MpiIdentity> for IdMatch {
    fn from(value: MpiIdentity) -> Self {
        IdMatch {
            idat: value.clone().into(),
            link_id: value.identity_id,
        }
    }
}

impl TryInto<Resource> for IdRequest {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Resource, Self::Error> {
        Ok(Resource::Patient(self.try_into()?))
    }
}

/// Convert from chrono::NaiveDate to time::Date for the FHIR model
pub(crate) fn parse_date(date: NaiveDate) -> Result<time::Date, ComponentRange> {
    time::Date::from_calendar_date(
        date.year(),
        time::Month::try_from(date.month() as u8)?,
        date.day() as u8,
    )
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::model::Idat;
    use crate::ttp::epix::model::{IdentityAddress, MpiIdentity};
    use chrono::NaiveDate;

    #[test]
    fn test_identity_into_idat() {
        let expected = Idat {
            first_name: "Max".to_string(),
            last_name: "Mustermann".to_string(),
            birth_date: NaiveDate::from_ymd_opt(1981, 11, 2).unwrap(),
            birth_place: "Berlin".to_string(),
            birth_name: Some("Muster".to_string()),
            postal_code: "35037".to_string(),
            city: "Marburg".to_string(),
        };
        let identity = MpiIdentity {
            birth_date: NaiveDate::from_ymd_opt(1981, 11, 2).unwrap(),
            birth_place: "Berlin".to_string(),
            first_name: "Max".to_string(),
            last_name: "Mustermann".to_string(),
            mothers_maiden_name: Some("Muster".to_string()),
            contacts: IdentityAddress {
                zip_code: "35037".to_string(),
                city: "Marburg".to_string(),
            },
            identity_id: 0,
        };

        let actual: Idat = identity.into();

        assert_eq!(expected, actual);
    }
}
