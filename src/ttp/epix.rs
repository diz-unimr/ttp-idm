pub(crate) mod soap;

use crate::ttp::epix::soap::GetPossibleMatchesForPersonResponseEnvelope;
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
pub(crate) struct GetPossibleMatchesForPersonEnvelope {
    #[serde(rename = "soap:Body")]
    pub(crate) body: GetPossibleMatchesForPersonBody,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct GetPossibleMatchesForPersonBody {
    #[serde(rename = "ser:getPossibleMatchesForPerson")]
    pub(crate) get_possible_matches_for_person: GetPossibleMatchesForPerson,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetPossibleMatchesForPerson {
    pub(crate) domain_name: String,
    pub(crate) mpi_id: String,
}

// todo: variable soap body
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

impl TryInto<String> for GetPossibleMatchesForPersonEnvelope {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<String, Self::Error> {
        Ok(serde_config().to_string(&self)?)
    }
}

impl TryInto<String> for GetPossibleMatchesForPersonResponseEnvelope {
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

pub(crate) fn create_possible_matches_for_person_request(
    domain: String,
    mpi: String,
) -> GetPossibleMatchesForPersonEnvelope {
    GetPossibleMatchesForPersonEnvelope {
        body: GetPossibleMatchesForPersonBody {
            get_possible_matches_for_person: GetPossibleMatchesForPerson {
                domain_name: domain,
                mpi_id: mpi,
            },
        },
    }
}

fn serde_config() -> SerdeXml {
    serde_xml_rs::SerdeXml::new()
        .namespace("ser", "http://service.epix.ttp.icmvc.emau.org/")
        .namespace("ns2", "http://service.epix.ttp.icmvc.emau.org/")
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
    use crate::ttp::epix::soap::{
        GetPossibleMatchesForPersonResponse, GetPossibleMatchesForPersonResponseBody,
        GetPossibleMatchesForPersonResponseEnvelope, GetPossibleMatchesForPersonResponseReturn,
        IdentityAddress, MatchingIdentity, MpiIdentity,
    };
    use chrono::NaiveDate;

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

    #[tokio::test]
    async fn test_get_possible_matches_for_person_response() {
        let soap = r#"<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
    <soap:Body>
        <ns2:getPossibleMatchesForPersonResponse xmlns:ns2="http://service.epix.ttp.icmvc.emau.org/">
            <return>
                <creationType>OPEN</creationType>
                <linkId>42</linkId>
                <possibleMatchCreated>2025-11-21T14:16:23.242+01:00</possibleMatchCreated>
                <priority>OPEN</priority>
                <probability>3.1481482315455955</probability>
                <assignedIdentity>
                    <birthDate>1972-01-01T00:00:00+01:00</birthDate>
                    <birthPlace>Berlin</birthPlace>
                    <firstName>Erika</firstName>
                    <lastName>Mustermann</lastName>
                    <mothersMaidenName>Musterfrau</mothersMaidenName>
                    <deactivated>false</deactivated>
                    <identityCreated>2025-11-21T14:16:23.242+01:00</identityCreated>
                    <identityId>2</identityId>
                    <identityLastEdited>2025-11-21T14:16:23.242+01:00</identityLastEdited>
                    <identityVersion>1</identityVersion>
                    <personId>2</personId>
                    <source>
                        <description>dummy because of the default-property "safe_source" in table domain</description>
                        <entryDate>2025-11-19T23:11:27.909+01:00</entryDate>
                        <label>dummy_safe_source</label>
                        <name>dummy_safe_source</name>
                        <updateDate>2025-11-19T23:11:27.909+01:00</updateDate>
                    </source>
                    <contacts>
                        <city>Marburg</city>
                        <zipCode>35037</zipCode>
                        <contactCreated>2025-11-21T14:16:23.242+01:00</contactCreated>
                        <contactId>2</contactId>
                        <contactLastEdited>2025-11-21T14:16:23.242+01:00</contactLastEdited>
                        <contactVersion>1</contactVersion>
                        <deactivated>false</deactivated>
                        <identityId>1</identityId>
                    </contacts>
                </assignedIdentity>
                <matchingMPIIdentity>
                    <identity>
                        <birthDate>1972-01-01T00:00:00+01:00</birthDate>
                        <birthPlace>Berlin</birthPlace>
                        <firstName>Erika</firstName>
                        <lastName>Mustermann</lastName>
                        <mothersMaidenName>Musterfrau</mothersMaidenName>
                        <deactivated>false</deactivated>
                        <identityCreated>2025-11-19T23:12:53.361+01:00</identityCreated>
                        <identityId>1</identityId>
                        <identityLastEdited>2025-11-19T23:12:53.361+01:00</identityLastEdited>
                        <identityVersion>1</identityVersion>
                        <personId>1</personId>
                        <source>
                            <description>dummy because of the default-property "safe_source" in table domain</description>
                            <entryDate>2025-11-19T23:11:27.909+01:00</entryDate>
                            <label>dummy_safe_source</label>
                            <name>dummy_safe_source</name>
                            <updateDate>2025-11-19T23:11:27.909+01:00</updateDate>
                        </source>
                        <contacts>
                            <city>Marburg</city>
                            <zipCode>35037</zipCode>
                            <contactCreated>2025-11-19T23:12:53.361+01:00</contactCreated>
                            <contactId>1</contactId>
                            <contactLastEdited>2025-11-19T23:12:53.361+01:00</contactLastEdited>
                            <contactVersion>1</contactVersion>
                            <deactivated>false</deactivated>
                            <identityId>1</identityId>
                        </contacts>
                    </identity>
                    <mpiId>
                        <description>generated MPI id</description>
                        <entryDate>2025-11-19T23:12:53.361+01:00</entryDate>
                        <fresh>false</fresh>
                        <identifierDomain>
                            <entryDate>2025-11-19T23:11:27.545+01:00</entryDate>
                            <label>MPI</label>
                            <name>MPI</name>
                            <oid>1.2.276.0.76.3.1.132.1.1.1</oid>
                            <updateDate>2025-11-19T23:11:27.545+01:00</updateDate>
                        </identifierDomain>
                        <value>1001000000001</value>
                    </mpiId>
                </matchingMPIIdentity>
                <requestedMPI>
                    <description>generated MPI id</description>
                    <entryDate>2025-11-21T14:16:23.242+01:00</entryDate>
                    <fresh>false</fresh>
                    <identifierDomain>
                        <entryDate>2025-11-19T23:11:27.545+01:00</entryDate>
                        <label>MPI</label>
                        <name>MPI</name>
                        <oid>1.2.276.0.76.3.1.132.1.1.1</oid>
                        <updateDate>2025-11-19T23:11:27.545+01:00</updateDate>
                    </identifierDomain>
                    <value>1001000000002</value>
                </requestedMPI>
            </return>
        </ns2:getPossibleMatchesForPersonResponse>
    </soap:Body>
</soap:Envelope>"#;

        let matches_body = GetPossibleMatchesForPersonResponseEnvelope {
            body: GetPossibleMatchesForPersonResponseBody {
                get_possible_matches_for_person_response: GetPossibleMatchesForPersonResponse {
                    returns: vec![GetPossibleMatchesForPersonResponseReturn {
                        link_id: 42,
                        priority: "OPEN".to_string(),
                        matching_identity: MatchingIdentity {
                            identity: MpiIdentity {
                                birth_date: NaiveDate::from_ymd_opt(1972, 1, 1).unwrap(),
                                birth_place: "Berlin".to_string(),
                                first_name: "Erika".to_string(),
                                last_name: "Mustermann".to_string(),
                                mothers_maiden_name: Some("Musterfrau".into()),
                                contacts: IdentityAddress {
                                    zip_code: "35037".to_string(),
                                    city: "Marburg".to_string(),
                                },
                            },
                        },
                    }],
                },
            },
        };

        let reverse = GetPossibleMatchesForPersonResponseEnvelope::try_from(soap).unwrap();
        assert_eq!(matches_body, reverse);
    }
}
