use anyhow::anyhow;
use fhir_model::r4b::resources::{Parameters, ParametersParameter, ParametersParameterValue};
use fhir_model::BuilderError;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename = "soap:Envelope")]
pub(crate) struct AddDomainEnvelope {
    #[serde(rename = "soap:Body")]
    body: AddDomainBody,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct AddDomainBody {
    #[serde(rename = "psn:addDomain")]
    add_domain: AddDomain,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct AddDomain {
    #[serde(rename = "domainDTO")]
    domain: Domain,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
struct Domain {
    name: String,
    label: Option<String>,
    check_digit_class: String,
    alphabet: String,
    parent_domain_names: Option<String>,
    config: DomainConfig,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
struct DomainConfig {
    psn_length: i32,
    psn_prefix: Option<String>,
    psns_deletable: bool,
    multi_psn_domain: bool,
    send_notifications_web: bool,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename = "soap:Envelope")]
pub(crate) struct GetPseudonymsForEnvelope {
    #[serde(rename = "soap:Body")]
    body: GetPseudonymsForBody,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct GetPseudonymsForBody {
    #[serde(rename = "psn:getPseudonymsFor")]
    get_pseudonyms_for: GetPseudonymsFor,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
struct GetPseudonymsFor {
    value: String,
    domain_name: String,
}

impl TryInto<String> for AddDomainEnvelope {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<String, Self::Error> {
        let config = serde_xml_rs::SerdeXml::new()
            .namespace("psn", "http://psn.ttp.ganimed.icmvc.emau.org/")
            .namespace("soap", "http://schemas.xmlsoap.org/soap/envelope/");

        Ok(config.to_string(&self)?)
    }
}

pub(crate) fn create_domain_request(
    domain: String,
    label: Option<String>,
    prefix: Option<String>,
    is_multi_psn: bool,
    parent_domain: Option<String>,
) -> AddDomainEnvelope {
    AddDomainEnvelope {
        body: AddDomainBody {
            add_domain: AddDomain {
                domain: Domain {
                    name: domain,
                    label,
                    check_digit_class: "org.emau.icmvc.ganimed.ttp.psn.generator.NoCheckDigits"
                        .into(),
                    alphabet: "org.emau.icmvc.ganimed.ttp.psn.alphabets.Symbol32".into(),
                    parent_domain_names: parent_domain,
                    config: DomainConfig {
                        psn_length: 16,
                        psn_prefix: prefix,
                        psns_deletable: false,
                        multi_psn_domain: is_multi_psn,
                        send_notifications_web: true,
                    },
                },
            },
        },
    }
}

pub(crate) fn create_psn_request(
    domain: String,
    value: String,
) -> Result<Parameters, BuilderError> {
    Parameters::builder()
        .parameter(vec![
            Some(
                ParametersParameter::builder()
                    .name("target".to_string())
                    .value(ParametersParameterValue::String(domain))
                    .build()?,
            ),
            Some(
                ParametersParameter::builder()
                    .name("original".to_string())
                    .value(ParametersParameterValue::String(value))
                    .build()?,
            ),
        ])
        .build()
}

pub(crate) fn get_secondary_psn_request(domain: String, value: String) -> GetPseudonymsForEnvelope {
    GetPseudonymsForEnvelope {
        body: GetPseudonymsForBody {
            get_pseudonyms_for: GetPseudonymsFor {
                value,
                domain_name: domain,
            },
        },
    }
}

pub(crate) fn create_secondary_psn_request(
    domain: String,
    value: String,
    count: String,
) -> Result<Parameters, BuilderError> {
    Parameters::builder()
        .parameter(vec![Some(
            ParametersParameter::builder()
                .name("original".to_string())
                .part(vec![
                    Some(
                        ParametersParameter::builder()
                            .name("target".to_string())
                            .value(ParametersParameterValue::String(domain))
                            .build()?,
                    ),
                    Some(
                        ParametersParameter::builder()
                            .name("value".to_string())
                            .value(ParametersParameterValue::String(value))
                            .build()?,
                    ),
                    Some(
                        ParametersParameter::builder()
                            .name("count".to_string())
                            .value(ParametersParameterValue::String(count))
                            .build()?,
                    ),
                ])
                .build()?,
        )])
        .build()
}

pub(crate) fn parse_pseudonym(params: Parameters) -> anyhow::Result<String> {
    params
        .parameter
        .iter()
        .flatten()
        .filter_map(|p| {
            if p.name == "pseudonym" {
                Some(p.part.iter().flatten())
            } else {
                None
            }
        })
        .flatten()
        .filter_map(|p| match p.name.as_str() {
            "pseudonym" => Some(&p.value),
            _ => None,
        })
        .filter_map(|p| match &p {
            Some(ParametersParameterValue::Identifier(v)) => v.value.clone(),
            _ => None,
        })
        .next()
        .ok_or(anyhow!("Failed to parse pseudonym from gPAS response"))
}

pub(crate) fn parse_secondary(params: Parameters) -> Vec<String> {
    params
        .parameter
        .iter()
        .flatten()
        .filter_map(|p| {
            if p.name == "secondarypseudonym" {
                Some(p.part.iter().flatten())
            } else {
                None
            }
        })
        .flatten()
        .filter_map(|p| match p.name.as_str() {
            "value" => Some(p.value.clone().and_then(|v| match v {
                ParametersParameterValue::Identifier(v) => v.value.clone(),
                _ => None,
            })),
            _ => None,
        })
        .flatten()
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::ttp::client::FaultException::DomainInUse;
    use crate::ttp::client::{Fault, FaultBody, FaultEnvelope};
    use crate::ttp::gpas::create_domain_request;

    #[test]
    fn add_domain_envelope_test() {
        let soap = r#"
        <?xml version="1.0" encoding="UTF-8"?>
        <soap:Envelope xmlns:psn="http://psn.ttp.ganimed.icmvc.emau.org/" xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
            <soap:Body>
                <psn:addDomain>
                    <domainDTO>
                        <name>test</name>
                        <label>label</label>
                        <checkDigitClass>org.emau.icmvc.ganimed.ttp.psn.generator.NoCheckDigits</checkDigitClass>
                        <alphabet>org.emau.icmvc.ganimed.ttp.psn.alphabets.Symbol32</alphabet>
                        <parentDomainNames>parent</parentDomainNames>
                        <config>
                            <psnLength>16</psnLength>
                            <psnPrefix>PSN</psnPrefix>
                            <psnsDeletable>false</psnsDeletable>
                            <multiPsnDomain>true</multiPsnDomain>
                            <sendNotificationsWeb>true</sendNotificationsWeb>
                        </config>
                    </domainDTO>
                 </psn:addDomain>
            </soap:Body>
        </soap:Envelope>"#.trim();

        let add_domain = create_domain_request(
            "test".to_string(),
            Some("label".to_string()),
            Some("PSN".to_string()),
            true,
            Some("parent".to_string()),
        );

        let actual: String = add_domain.try_into().unwrap();

        let expected = soap
            .split("\n")
            .map(|s| s.trim())
            .collect::<Vec<&str>>()
            .join("")
            .to_string();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_gpas_fault_test() {
        let soap = r#"
        <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
            <soap:Body>
                <soap:Fault>
                    <faultcode>soap:Server</faultcode>
                    <faultstring>domain test already exists</faultstring>
                    <detail>
                        <ns1:DomainInUseException xmlns:ns1="http://psn.ttp.ganimed.icmvc.emau.org/"/>
                    </detail>
                </soap:Fault>
            </soap:Body>
        </soap:Envelope>"#;

        let env: FaultEnvelope = soap.to_string().try_into().unwrap();

        assert_eq!(
            env,
            FaultEnvelope {
                body: FaultBody {
                    fault: Fault {
                        faultcode: "soap:Server".to_string(),
                        faultstring: "domain test already exists".to_string(),
                        detail: DomainInUse(())
                    },
                }
            }
        );
    }
}
