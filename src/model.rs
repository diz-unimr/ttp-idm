use chrono::{Datelike, NaiveDate};
use fhir_model::r4b::codes::NameUse;
use fhir_model::r4b::resources::{Patient, Resource};
use fhir_model::r4b::types::{Address, Extension, ExtensionValue, HumanName, Meta};
use fhir_model::Date;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use time::error::ComponentRange;

#[derive(Deserialize, Serialize)]
pub(crate) struct IdResponse {
    pub(crate) patient_id: String,
    pub(crate) lab: HashMap<String, Vec<String>>,
}

#[derive(Deserialize, Clone)]
pub(crate) struct IdRequest {
    first_name: String,
    last_name: String,
    birth_date: chrono::NaiveDate,
    birth_place: String,
    birth_name: Option<String>,
    postal_code: String,
    city: String,
    pub(crate) study: String,
    pub(crate) lab: HashMap<String, u32>,
}

impl TryInto<Patient> for IdRequest {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Patient, Self::Error> {
        // first name, last name
        let mut names = vec![Some(
            HumanName::builder()
                .given(
                    self.first_name
                        .split(' ')
                        .map(|n| Some(n.to_string()))
                        .collect::<Vec<_>>(),
                )
                .family(self.last_name)
                .build()?,
        )];

        // (optional) birth name
        if let Some(name) = self.birth_name {
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
            .birth_date(Date::Date(parse_date(self.birth_date)?))
            .address(vec![Some(
                Address::builder()
                    .postal_code(self.postal_code)
                    .city(self.city)
                    .build()?,
            )])
            .extension(vec![
                Extension::builder()
                    .url("http://hl7.org/fhir/StructureDefinition/patient-birthPlace".to_string())
                    .value(ExtensionValue::Address(
                        Address::builder().city(self.birth_place).build()?,
                    ))
                    .build()?,
            ]);

        Ok(builder.build()?)
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
