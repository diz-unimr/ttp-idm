use chrono::NaiveDate;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename = "soap:Envelope")]
pub(crate) struct SoapEnvelope {
    #[serde(rename = "soap:Body")]
    pub(crate) body: SoapBody,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename = "soap:Envelope")]
pub(crate) struct SoapBody {
    #[serde(rename = "soap:Body")]
    pub(crate) service: SoapMethod,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum SoapMethod {
    PossibleMatches(GetPossibleMatchesForPersonResponseBody),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename = "soap:Envelope")]
pub(crate) struct GetPossibleMatchesForPersonResponseEnvelope {
    #[serde(rename = "soap:Body")]
    pub(crate) body: GetPossibleMatchesForPersonResponseBody,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct GetPossibleMatchesForPersonResponseBody {
    #[serde(rename = "ns2:getPossibleMatchesForPersonResponse")]
    pub(crate) get_possible_matches_for_person_response: GetPossibleMatchesForPersonResponse,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct GetPossibleMatchesForPersonResponse {
    #[serde(rename = "return")]
    pub(crate) returns: Vec<GetPossibleMatchesForPersonResponseReturn>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetPossibleMatchesForPersonResponseReturn {
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
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IdentityAddress {
    pub(crate) zip_code: String,
    pub(crate) city: String,
}

impl TryFrom<&str> for GetPossibleMatchesForPersonResponseEnvelope {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let config = serde_xml_rs::SerdeXml::new()
            .namespace("ns1", "http://service.epix.ttp.icmvc.emau.org/")
            .namespace("soap", "http://schemas.xmlsoap.org/soap/envelope/");

        let env: Self = config.from_str(value)?;
        Ok(env)
    }
}

mod naive_date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Deserializer, Serializer};

    // const FORMAT: &str = "%Y-%m-%d";
    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S%z";

    /// Transforms a NaiveDate into a String
    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.format(FORMAT).to_string();
        serializer.serialize_str(&s)
    }

    /// Transforms a String into a NaiveDate
    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}
