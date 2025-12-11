pub(crate) mod model;

use crate::ttp::client::SoapEnvelope;
pub(crate) use crate::ttp::gpas::model::{
    AddDomain, AddDomainBody, AddDomainEnvelope, Domain, DomainConfig, GetDomain, GetDomainBody,
    GetPseudonymsFor, GetPseudonymsForBody, PsnOperation,
};
use anyhow::anyhow;
use fhir_model::r4b::resources::{Parameters, ParametersParameter, ParametersParameterValue};
use fhir_model::BuilderError;

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
                    child_domain_names: None,
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

pub(crate) fn create_get_domain_request(trial: String) -> SoapEnvelope<GetDomainBody> {
    SoapEnvelope::new(GetDomainBody {
        get_domain: GetDomain { domain_name: trial },
    })
}

pub(crate) fn create_psn_request(
    domain: String,
    value: String,
    op: PsnOperation,
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
                    .name(op.into())
                    .value(ParametersParameterValue::String(value))
                    .build()?,
            ),
        ])
        .build()
}

pub(crate) fn get_secondary_psn_request(
    domain: String,
    value: String,
) -> SoapEnvelope<GetPseudonymsForBody> {
    SoapEnvelope::new(GetPseudonymsForBody {
        get_pseudonyms_for: GetPseudonymsFor {
            value,
            domain_name: domain,
        },
    })
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

pub(crate) fn parse_pseudonym(params: Parameters, target: &str) -> anyhow::Result<String> {
    params
        .parameter
        .iter()
        .flatten()
        .filter_map(|p| {
            if p.name == target {
                Some(p.part.iter().flatten())
            } else {
                None
            }
        })
        .flatten()
        .filter_map(|p| match p.name.as_str() {
            part if part == target => Some(&p.value),
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
        <soap:Envelope xmlns:ns2="http://psn.ttp.ganimed.icmvc.emau.org/" xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
            <soap:Body>
                <ns2:addDomain>
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
                 </ns2:addDomain>
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
