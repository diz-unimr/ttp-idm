use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename = "soap:Envelope")]
pub(crate) struct SoapEnvelope<T> {
    #[serde(rename = "soap:Body")]
    pub(crate) body: T,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct GetPossibleMatchesForPersonResponseBody {
    #[serde(rename = "ns2:getPossibleMatchesForPersonResponse")]
    pub(crate) get_possible_matches_for_person_response: GetPossibleMatchesForPersonResponse,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct GetPossibleMatchesForPersonResponse {
    #[serde(rename = "return")]
    pub(crate) returns: Vec<PossibleMatchResult>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PossibleMatchResult {
    pub(crate) link_id: u32,
    pub(crate) priority: String,
    #[serde(rename = "matchingMPIIdentity")]
    pub(crate) matching_identity: MatchingIdentity,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct MatchingIdentity {
    pub(crate) identity: MpiIdentity,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MpiIdentity {
    #[serde(with = "naive_date_format")]
    pub(crate) birth_date: NaiveDate,
    pub(crate) mothers_maiden_name: Option<String>,
    pub(crate) birth_place: String,
    pub(crate) first_name: String,
    pub(crate) last_name: String,
    pub(crate) contacts: IdentityAddress,
    pub(crate) identity_id: u32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IdentityAddress {
    pub(crate) zip_code: String,
    pub(crate) city: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) struct AddDomainBody {
    #[serde(rename = "ser:addDomain")]
    pub(super) add_domain: AddDomain,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(super) struct AddDomain {
    pub(super) domain: Domain,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(super) struct Domain {
    pub(super) name: String,
    pub(super) description: String,
    pub(super) label: String,
    pub(super) mpi_domain: MpiDomain,
    pub(super) safe_source: SafeSource,
    pub(super) config: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(super) struct MpiDomain {
    pub(super) name: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(super) struct SafeSource {
    pub(super) name: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct PossibleMatchesForPersonBody {
    #[serde(rename = "ser:getPossibleMatchesForPerson")]
    pub(super) get_possible_matches_for_person: PossibleMatchesForPerson,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct PossibleMatchesForPerson {
    pub(super) domain_name: String,
    pub(super) mpi_id: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct PossibleMatchesForDomainBody {
    #[serde(rename = "ser:getPossibleMatchesForDomain")]
    pub(crate) possible_matches_for_domain: PossibleMatchesForDomain,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PossibleMatchesForDomain {
    pub(super) domain_name: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct AddIdentifierDomainBody {
    #[serde(rename = "ser:addIdentifierDomain")]
    pub(super) add_identifier_domain: AddIdentifierDomain,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct AddIdentifierDomain {
    pub(super) identifier_domain: IdentifierDomain,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct IdentifierDomain {
    pub(super) name: String,
    pub(super) label: String,
    pub(super) oid: Uuid,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct AddDataSourceBody {
    #[serde(rename = "ser:addSource")]
    pub(super) add_source: AddDataSource,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(super) struct AddDataSource {
    pub(super) source: DataSource,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct DataSource {
    pub(super) name: String,
    pub(super) label: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct RemovePossibleMatchBody {
    #[serde(rename = "ser:removePossibleMatch")]
    pub(super) remove_possible_match: RemovePossibleMatch,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct RemovePossibleMatch {
    pub(super) possible_match_id: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct AssignIdentityBody {
    #[serde(rename = "ser:assignIdentity")]
    pub(super) assign_identity: AssignIdentity,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct AssignIdentity {
    pub(super) possible_match_id: u32,
    pub(crate) winning_identity_id: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct PossibleMatchesForDomainResponseBody {
    #[serde(rename = "ns2:getPossibleMatchesForDomainResponse")]
    pub(crate) possible_matches_for_domain_response: PossibleMatchesForDomainResponse,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct PossibleMatchesForDomainResponse {
    #[serde(rename = "return")]
    pub(crate) returns: Vec<PossibleMatchesForDomainResponseReturn>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PossibleMatchesForDomainResponseReturn {
    pub(crate) link_id: u32,
    pub(crate) priority: String,
    #[serde(rename = "matchingMPIIdentities")]
    pub(crate) matching_identities: Vec<MatchingIdentity>,
}

impl<T> SoapEnvelope<T> {
    pub(super) fn new(body: T) -> Self {
        SoapEnvelope::<T> { body }
    }
}

impl<'a, T: Deserialize<'a>> TryFrom<&str> for SoapEnvelope<T> {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // todo: remove config?
        let config = serde_xml_rs::SerdeXml::new()
            .namespace("ns1", "http://service.epix.ttp.icmvc.emau.org/")
            .namespace("ser", "http://service.epix.ttp.icmvc.emau.org/")
            .namespace("soap", "http://schemas.xmlsoap.org/soap/envelope/");

        let env: Self = config.from_str(value)?;
        Ok(env)
    }
}

impl<T: Serialize> TryInto<String> for SoapEnvelope<T> {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<String, Self::Error> {
        let config = serde_xml_rs::SerdeXml::new()
            .namespace("ns1", "http://service.epix.ttp.icmvc.emau.org/")
            .namespace("ser", "http://service.epix.ttp.icmvc.emau.org/")
            .namespace("soap", "http://schemas.xmlsoap.org/soap/envelope/");

        let env: String = config.to_string(&self)?;
        Ok(env)
    }
}

mod naive_date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S%z";

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.format(FORMAT).to_string();
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}
