use serde_derive::{Deserialize, Serialize};
use serde_xml_rs::SerdeXml;
use std::{env, fs};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
#[serde(rename = "soap:Envelope")]
pub(crate) struct AddDomainEnvelope {
    #[serde(rename = "soap:Body")]
    body: AddDomainBody,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct AddDomainBody {
    #[serde(rename = "ser:addDomain")]
    add_domain: AddDomain,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct AddDomain {
    domain: Domain,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Domain {
    name: String,
    description: String,
    label: String,
    mpi_domain: MpiDomain,
    safe_source: SafeSource,
    config: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct MpiDomain {
    name: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct SafeSource {
    name: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename = "soap:Envelope")]
pub(crate) struct AddIdentifierDomainEnvelope {
    #[serde(rename = "soap:Body")]
    body: AddIdentifierDomainBody,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct AddIdentifierDomainBody {
    #[serde(rename = "ser:addIdentifierDomain")]
    add_identifier_domain: AddIdentifierDomain,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AddIdentifierDomain {
    identifier_domain: IdentifierDomain,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
struct IdentifierDomain {
    name: String,
    label: String,
    oid: Uuid,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename = "soap:Envelope")]
pub(crate) struct AddDataSourceEnvelope {
    #[serde(rename = "soap:Body")]
    body: AddDataSourceBody,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct AddDataSourceBody {
    #[serde(rename = "ser:addSource")]
    add_source: AddDataSource,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct AddDataSource {
    source: DataSource,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
struct DataSource {
    name: String,
    label: String,
}

impl TryInto<String> for AddDomainEnvelope {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<String, Self::Error> {
        Ok(serde_config().to_string(&self)?)
    }
}

impl TryInto<String> for AddIdentifierDomainEnvelope {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<String, Self::Error> {
        Ok(serde_config().to_string(&self)?)
    }
}

impl TryInto<String> for AddDataSourceEnvelope {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<String, Self::Error> {
        Ok(serde_config().to_string(&self)?)
    }
}

pub(crate) fn create_domain_request(
    domain: String,
    description: String,
    mpi_domain: String,
    safe_source: String,
) -> Result<AddDomainEnvelope, anyhow::Error> {
    Ok(AddDomainEnvelope {
        body: AddDomainBody {
            add_domain: AddDomain {
                domain: Domain {
                    name: domain.clone(),
                    description,
                    label: domain,
                    mpi_domain: MpiDomain { name: mpi_domain },
                    safe_source: SafeSource { name: safe_source },
                    config: load_matching_config()?,
                },
            },
        },
    })
}

pub(crate) fn create_id_domain_request(domain: String) -> AddIdentifierDomainEnvelope {
    AddIdentifierDomainEnvelope {
        body: AddIdentifierDomainBody {
            add_identifier_domain: AddIdentifierDomain {
                identifier_domain: IdentifierDomain {
                    name: domain.clone(),
                    label: domain,
                    oid: Uuid::new_v4(),
                },
            },
        },
    }
}

pub(crate) fn create_data_source_request(source: String) -> AddDataSourceEnvelope {
    AddDataSourceEnvelope {
        body: AddDataSourceBody {
            add_source: AddDataSource {
                source: DataSource {
                    name: source.clone(),
                    label: source,
                },
            },
        },
    }
}

fn serde_config() -> SerdeXml {
    serde_xml_rs::SerdeXml::new()
        .namespace("ser", "http://service.epix.ttp.icmvc.emau.org/")
        .namespace("soap", "http://schemas.xmlsoap.org/soap/envelope/")
}

fn load_matching_config() -> Result<String, anyhow::Error> {
    // get resource dir
    let base_dir = env::current_dir()?.join("resources");

    let matching_config_path = base_dir.join("matching_config.xml");
    let config = fs::read_to_string(matching_config_path)?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use crate::ttp::client::FaultException::DuplicateEntryException;
    use crate::ttp::client::{Fault, FaultBody, FaultEnvelope};

    #[test]
    fn parse_epix_fault_response_test() {
        let soap = r#"
        <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
            <soap:Body>
                <soap:Fault>
                    <faultcode>soap:Server</faultcode>
                    <faultstring>identifier domain already exists: Test</faultstring>
                    <detail>
                        <ns1:DuplicateEntryException xmlns:ns1="http://service.epix.ttp.icmvc.emau.org/"/>
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
                        faultstring: "identifier domain already exists: Test".to_string(),
                        detail: DuplicateEntryException(())
                    },
                }
            }
        );
    }
}
