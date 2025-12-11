use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename = "soap:Envelope")]
pub(crate) struct AddDomainEnvelope {
    #[serde(rename = "soap:Body")]
    pub(crate) body: AddDomainBody,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct AddDomainBody {
    #[serde(rename = "ns2:addDomain")]
    pub(crate) add_domain: AddDomain,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct AddDomain {
    #[serde(rename = "domainDTO")]
    pub(crate) domain: Domain,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Domain {
    pub(crate) name: String,
    pub(crate) label: Option<String>,
    pub(crate) check_digit_class: String,
    pub(crate) alphabet: String,
    pub(crate) parent_domain_names: Option<String>,
    pub(crate) child_domain_names: Option<Vec<String>>,
    pub(crate) config: DomainConfig,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DomainConfig {
    pub(crate) psn_length: i32,
    pub(crate) psn_prefix: Option<String>,
    pub(crate) psns_deletable: bool,
    pub(crate) multi_psn_domain: bool,
    pub(crate) send_notifications_web: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct GetDomainBody {
    #[serde(rename = "ns2:getDomain")]
    pub(crate) get_domain: GetDomain,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetDomain {
    pub(crate) domain_name: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct GetDomainResponseBody {
    #[serde(rename = "ns2:getDomainResponse")]
    pub(crate) get_domain_response: GetDomainResponse,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct GetDomainResponse {
    pub(crate) domain: Domain,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct GetPseudonymsForBody {
    #[serde(rename = "ns2:getPseudonymsFor")]
    pub(crate) get_pseudonyms_for: GetPseudonymsFor,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetPseudonymsFor {
    pub(crate) value: String,
    pub(crate) domain_name: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct GetPseudonymsForResponseBody {
    #[serde(rename = "ns2:getPseudonymsForResponse")]
    pub(crate) get_pseudonyms_for_response: GetPseudonymsForResponse,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct GetPseudonymsForResponse {
    #[serde(rename = "return")]
    pub(crate) returns: GetPseudonymsForResponseReturn,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct GetPseudonymsForResponseReturn {
    pub(crate) psn: Vec<String>,
}

pub(crate) enum PsnOperation {
    Pseudonymize,
    Identify,
}

impl Into<String> for PsnOperation {
    fn into(self) -> String {
        match self {
            PsnOperation::Pseudonymize => "original".into(),
            PsnOperation::Identify => "pseudonym".into(),
        }
    }
}

impl TryInto<String> for AddDomainEnvelope {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<String, Self::Error> {
        let config = serde_xml_rs::SerdeXml::new()
            .namespace("ns2", "http://psn.ttp.ganimed.icmvc.emau.org/")
            .namespace("soap", "http://schemas.xmlsoap.org/soap/envelope/");

        Ok(config.to_string(&self)?)
    }
}
