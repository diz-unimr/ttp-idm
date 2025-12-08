pub(crate) mod model;

use crate::ttp::client::SoapEnvelope;
use crate::ttp::epix::model::{
    AddDataSource, AddDataSourceBody, AddDomain, AddDomainBody, AddIdentifierDomain,
    AddIdentifierDomainBody, AssignIdentity, AssignIdentityBody, DataSource, Domain,
    IdentifierDomain, MpiDomain, PossibleMatchesForDomain, PossibleMatchesForDomainBody,
    PossibleMatchesForPerson, PossibleMatchesForPersonBody, RemovePossibleMatch,
    RemovePossibleMatchBody, SafeSource,
};
use std::{env, fs};
use uuid::Uuid;

pub(crate) fn create_domain_request(
    domain: String,
    description: String,
    mpi_domain: String,
    safe_source: String,
) -> Result<SoapEnvelope<AddDomainBody>, anyhow::Error> {
    Ok(SoapEnvelope::new(AddDomainBody {
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
    }))
}

pub(crate) fn id_domain_request(domain: String) -> SoapEnvelope<AddIdentifierDomainBody> {
    SoapEnvelope::new(AddIdentifierDomainBody {
        add_identifier_domain: AddIdentifierDomain {
            identifier_domain: IdentifierDomain {
                name: domain.clone(),
                label: domain,
                oid: Uuid::new_v4(),
            },
        },
    })
}

pub(crate) fn data_source_request(source: String) -> SoapEnvelope<AddDataSourceBody> {
    SoapEnvelope::new(AddDataSourceBody {
        add_source: AddDataSource {
            source: DataSource {
                name: source.clone(),
                label: source,
            },
        },
    })
}

pub(crate) fn remove_possible_match_request(
    match_id: u32,
) -> SoapEnvelope<RemovePossibleMatchBody> {
    SoapEnvelope::new(RemovePossibleMatchBody {
        remove_possible_match: RemovePossibleMatch {
            possible_match_id: match_id,
        },
    })
}

pub(crate) fn possible_matches_for_person_request(
    domain: String,
    mpi: String,
) -> SoapEnvelope<PossibleMatchesForPersonBody> {
    SoapEnvelope::new(PossibleMatchesForPersonBody {
        get_possible_matches_for_person: PossibleMatchesForPerson {
            domain_name: domain,
            mpi_id: mpi,
        },
    })
}

pub(crate) fn possible_matches_for_domain_request(
    domain: String,
) -> SoapEnvelope<PossibleMatchesForDomainBody> {
    SoapEnvelope::new(PossibleMatchesForDomainBody {
        possible_matches_for_domain: PossibleMatchesForDomain {
            domain_name: domain,
        },
    })
}

pub(crate) fn assign_identity_request(
    possible_match_id: u32,
    winning_identity_id: u32,
) -> SoapEnvelope<AssignIdentityBody> {
    SoapEnvelope::new(AssignIdentityBody {
        assign_identity: AssignIdentity {
            possible_match_id,
            winning_identity_id,
        },
    })
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
    use crate::ttp::client::FaultException::DuplicateEntry;
    use crate::ttp::client::{Fault, FaultBody, FaultEnvelope, SoapEnvelope};
    use crate::ttp::epix::model::{
        GetPossibleMatchesForPersonResponse, GetPossibleMatchesForPersonResponseBody,
        IdentityAddress, MatchingIdentity, MpiIdentity, PossibleMatchResult,
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
                        detail: DuplicateEntry(())
                    },
                }
            }
        );
    }

    #[tokio::test]
    async fn test_get_possible_matches_for_person_response() {
        let soap = r#"<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
    <soap:Body>
        <ns1:getPossibleMatchesForPersonResponse xmlns:ns1="http://service.epix.ttp.icmvc.emau.org/">
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
        </ns1:getPossibleMatchesForPersonResponse>
    </soap:Body>
</soap:Envelope>"#;

        let matches = SoapEnvelope::new(GetPossibleMatchesForPersonResponseBody {
            get_possible_matches_for_person_response: GetPossibleMatchesForPersonResponse {
                returns: vec![PossibleMatchResult {
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
                            identity_id: 1,
                        },
                    },
                }],
            },
        });

        let reverse = SoapEnvelope::try_from(soap).unwrap();
        assert_eq!(matches, reverse);
    }
}
