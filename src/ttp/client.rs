use crate::api::IdRequest;
use crate::config::{Epix, Gpas, Ttp};
use crate::ttp::epix::model::{GetPossibleMatchesForPersonResponseBody, PossibleMatchResult};
use crate::ttp::gpas::model::{GetDomainResponseBody, GetPseudonymsForResponseBody};
use crate::ttp::gpas::PsnOperation;
use crate::ttp::{epix, gpas};
use anyhow::anyhow;
use fhir_model::r4b::resources::{Parameters, ParametersParameter, ParametersParameterValue};
use fhir_model::r4b::types::Coding;
use log::{debug, error, info, warn};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{header, Client, Error, Response};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinSet;

#[derive(Debug, Clone)]
pub(crate) struct TtpClient {
    client: Client,
    epix: Epix,
    gpas: Gpas,
}

impl TtpClient {
    pub(crate) async fn setup_domains(&self) -> Result<(), anyhow::Error> {
        // epix
        self.setup_epix_domains().await?;

        Ok(())
    }

    async fn setup_epix_domains(&self) -> anyhow::Result<()> {
        // create identifier domain
        let soap = epix::id_domain_request(self.epix.identifier_domain.to_string());
        self.create_epix_domain(soap.try_into()?).await?;

        // create data source
        let soap = epix::data_source_request(self.epix.data_source.to_string());
        self.create_epix_domain(soap.try_into()?).await?;

        // epix study domain
        let soap = epix::create_domain_request(
            self.epix.domain.name.to_string(),
            self.epix.domain.description.to_string(),
            self.epix.identifier_domain.to_string(),
            self.epix.data_source.to_string(),
        )?;

        let body = soap.try_into()?;
        self.create_epix_domain(body).await?;

        Ok(())
    }

    async fn setup_gpas_domains(
        &self,
        study: &str,
        lab: &HashMap<String, u32>,
    ) -> anyhow::Result<()> {
        // gpas
        // primary domain
        let soap_request = gpas::create_domain_request(study.to_string(), None, None, false, None);
        let body: String = soap_request.try_into()?;
        self.create_gpas_domain(body).await?;

        // lab (sub) domains
        for l in lab.keys() {
            let soap_request = gpas::create_domain_request(
                format!("{study}_{l}"),
                None,
                None,
                true,
                Some(study.to_string()),
            );
            let body: String = soap_request.try_into()?;
            self.create_gpas_domain(body).await?;
        }

        Ok(())
    }

    async fn create_gpas_domain(&self, body: String) -> Result<(), anyhow::Error> {
        let request = self
            .client
            .post(format!("{}/gpas/DomainService?wsdl", self.gpas.base_url).as_str())
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/soap+xml"),
            )
            .body(body);

        let response = request.send().await?;

        // if no success, check if already created
        if !response.status().is_success() {
            let resp_text = response.text().await?;
            FaultEnvelope::try_from(resp_text)?;
        }

        Ok(())
    }

    async fn create_epix_domain(&self, body: String) -> anyhow::Result<()> {
        let request = self
            .client
            .post(format!("{}/epix/epixManagementService?wsdl", self.epix.base_url).as_str())
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/soap+xml"),
            )
            .body(body);

        let response = request.send().await?;

        // if no success, check if already created
        if !response.status().is_success() {
            let resp_text = response.text().await?;
            let fault = FaultEnvelope::try_from(resp_text.clone())
                .map_err(|_| anyhow!("Failed to create E-PIX domain: {resp_text}"))?;
            debug!("E-PIX {}", fault.body.fault.faultstring);
        }

        Ok(())
    }

    pub(crate) async fn delete_identity(&self, identity_id: u32) -> anyhow::Result<()> {
        // deactivate first
        let body: String = epix::deactivate_entity_request(identity_id).try_into()?;
        let response = self.send_epix(body).await?;

        // todo: refactor check response
        if !response.status().is_success() {
            let resp_text = response.text().await?;
            let fault = FaultEnvelope::try_from(resp_text.clone())
                .map_err(|_| anyhow!("Failed to deactivate E-PIX identity: {}", resp_text))?;

            return Err(anyhow!(
                "Failed to deactivate E-PIX identity: {}",
                fault.body.fault.faultstring
            ));
        }
        debug!("E-PIX identity with id: {identity_id} successfully deactivated");

        // delete identity
        let body: String = epix::delete_entity_request(identity_id).try_into()?;
        let response = self.send_epix(body).await?;
        // todo check response
        if !response.status().is_success() {
            let resp_text = response.text().await?;
            let fault = FaultEnvelope::try_from(resp_text.clone())
                .map_err(|_| anyhow!("Failed to delete E-PIX identity: {}", resp_text))?;

            return Err(anyhow!(
                "Failed to delete E-PIX identity: {}",
                fault.body.fault.faultstring
            ));
        }

        debug!("E-PIX identity with id: {identity_id} successfully deleted");

        Ok(())
    }

    pub(crate) async fn possible_matches_for_person(
        &self,
        mpi: String,
    ) -> anyhow::Result<Vec<PossibleMatchResult>> {
        let body: String =
            epix::possible_matches_for_person_request(self.epix.domain.name.clone(), mpi)
                .try_into()?;

        let request = self
            .client
            .post(format!("{}/epix/epixService?wsdl", self.epix.base_url).as_str())
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/soap+xml"),
            )
            .body(body);

        let response = request.send().await?;
        let resp_body = response.text().await?;
        let matched =
            SoapEnvelope::<GetPossibleMatchesForPersonResponseBody>::try_from(resp_body.as_str())?;

        Ok(matched
            .body
            .get_possible_matches_for_person_response
            .returns)
    }

    pub(crate) async fn split_identities(&self, link_id: u32) -> anyhow::Result<()> {
        let body: String = epix::remove_possible_match_request(link_id).try_into()?;

        let request = self
            .client
            .post(format!("{}/epix/epixService?wsdl", self.epix.base_url).as_str())
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/soap+xml"),
            )
            .body(body);

        let response = request.send().await?;
        let status = response.status();

        if !status.is_success() {
            let fault = FaultEnvelope::try_from(response.text().await?);
            return Err(anyhow!(
                fault
                    .map(|f: FaultEnvelope| f.body.fault.faultstring)
                    .map_err(|e| {
                        warn!("E-PIX removePossibleMatch with id: {link_id} failed. {e}");
                        anyhow!("Failed to resolve possible E-PIX match with link id: {link_id}")
                    })?
            ));
        }

        Ok(())
    }

    pub(crate) async fn new(config: &Ttp) -> Result<Self, anyhow::Error> {
        // default headers
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/fhir+json"),
        );

        // http client
        let client = Client::builder()
            .default_headers(headers.clone())
            .timeout(Duration::from_secs(config.timeout))
            .build()?;

        Ok(TtpClient {
            client,
            epix: config.epix.clone(),
            gpas: config.gpas.clone(),
        })
    }

    pub(crate) async fn test_connection(&self) -> anyhow::Result<()> {
        // test epix
        self.get_metadata(format!("{}/ttp-fhir/fhir/epix", self.epix.base_url).as_str())
            .await?;
        info!("Connection test to E-PIX successful");

        // test gpas
        self.get_metadata(format!("{}/ttp-fhir/fhir/gpas", self.gpas.base_url).as_str())
            .await?;
        info!("Connection test to gPAS successful");

        Ok(())
    }

    async fn get_metadata(&self, base_url: &str) -> anyhow::Result<()> {
        let metadata = format!("{}/metadata", base_url);
        match self.client.get(metadata.as_str()).send().await {
            Ok(resp) => {
                if resp.status().is_success() {
                    Ok(())
                } else {
                    Err(anyhow!(
                        "Metadata response returned error code: {}",
                        resp.status()
                    ))
                }
            }

            Err(e) => {
                error!("Connection failed, {e}");
                Err(e.into())
            }
        }
    }

    pub(crate) async fn add_person(&self, idat: IdRequest) -> Result<Parameters, anyhow::Error> {
        let body = Parameters::builder()
            .parameter(vec![
                Some(
                    ParametersParameter::builder()
                        .name("domain".to_string())
                        .value(ParametersParameterValue::String(
                            self.epix.domain.name.clone(),
                        ))
                        .build()?,
                ),
                Some(
                    ParametersParameter::builder()
                        .name("source".to_string())
                        .value(ParametersParameterValue::String(
                            self.epix.data_source.clone(),
                        ))
                        .build()?,
                ),
                Some(
                    ParametersParameter::builder()
                        .name("saveAction".to_string())
                        .value(ParametersParameterValue::Coding(
                            Coding::builder()
                                .system(
                                    "https://ths-greifswald.de/fhir/CodeSystem/epix/SaveAction"
                                        .to_string(),
                                )
                                // .code(idat.on_match.save_action().to_string())
                                .code("DONT_SAVE_ON_PERFECT_MATCH_EXCEPT_CONTACTS".to_string())
                                .build()?,
                        ))
                        .build()?,
                ),
                Some(
                    ParametersParameter::builder()
                        .name("identity".to_string())
                        .resource(idat.try_into()?)
                        .build()?,
                ),
                Some(
                    ParametersParameter::builder()
                        .name("forceReferenceUpdate".to_string())
                        .value(ParametersParameterValue::Boolean(false))
                        .build()?,
                ),
            ])
            .build()?;

        let request = self
            .client
            .post(format!("{}/ttp-fhir/fhir/epix/$addPatient", self.epix.base_url).as_str())
            .body(serde_json::to_string(&body)?);

        let response = request.send().await?;

        Ok(serde_json::from_str(response.text().await?.as_str())?)
    }

    pub(crate) async fn pseudonymize(
        &self,
        mpi: String,
        id_request: IdRequest,
    ) -> Result<(String, HashMap<String, Vec<String>>), anyhow::Error> {
        // create study domains
        self.setup_gpas_domains(&id_request.trial, &id_request.lab)
            .await?;

        // pseudonymize mpi
        let mpi_psn = self
            .pseudonymize_mpi(id_request.trial.clone(), mpi.clone())
            .await?;

        // pseudonymize lab ids with mpi value
        let mut lab_ids = HashMap::<String, Vec<String>>::new();
        for (domain, count) in &id_request.lab {
            if *count > 0 {
                let ids = self
                    .pseudonymize_secondary(
                        &id_request.trial,
                        domain,
                        mpi.clone(),
                        count.to_string(),
                    )
                    .await?;
                lab_ids.insert(domain.clone(), ids);
            }
        }

        Ok((mpi_psn, lab_ids))
    }

    async fn pseudonymize_mpi(&self, study: String, mpi: String) -> anyhow::Result<String> {
        let body = gpas::create_psn_request(study, mpi, PsnOperation::Pseudonymize)?;
        let request = self
            .client
            .post(
                format!(
                    "{}/ttp-fhir/fhir/gpas/$pseudonymizeAllowCreate",
                    self.gpas.base_url
                )
                .as_str(),
            )
            .body(serde_json::to_string(&body)?);

        let response = request.send().await?;
        let params = serde_json::from_str(response.text().await?.as_str())?;
        gpas::parse_pseudonym(params, "pseudonym")
    }

    pub(crate) async fn identify(&self, domain: String, psn: String) -> anyhow::Result<String> {
        let body = gpas::create_psn_request(domain, psn, PsnOperation::Identify)?;
        let request = self
            .client
            .post(format!("{}/ttp-fhir/fhir/gpas/$dePseudonymize", self.gpas.base_url).as_str())
            .body(serde_json::to_string(&body)?);

        let response = request.send().await?;
        let params = serde_json::from_str(response.text().await?.as_str())?;
        gpas::parse_pseudonym(params, "original")
    }

    pub(crate) async fn get_pseudonyms(
        self: Arc<Self>,
        domains: Vec<String>,
        mpi: String,
    ) -> anyhow::Result<HashMap<String, Vec<String>>> {
        let mut set = JoinSet::new();
        domains.into_iter().for_each(|d| {
            let bla = Arc::clone(&self);
            let mpi = mpi.clone();
            set.spawn(async move { bla.get_pseudonyms_for_domain(d, mpi).await });
        });

        let psns = set
            .join_all()
            .await
            .into_iter()
            .flatten()
            .collect::<HashMap<String, Vec<String>>>();

        Ok(psns)
    }

    async fn get_pseudonyms_for_domain(
        &self,
        domain: String,
        value: String,
    ) -> anyhow::Result<(String, Vec<String>)> {
        // get trial domain
        let body: String = gpas::get_secondary_psn_request(domain.clone(), value).try_into()?;
        let request = self
            .client
            .post(format!("{}/gpas/gpasService?wsdl", self.gpas.base_url).as_str())
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/soap+xml"),
            )
            .body(body);

        let response = request.send().await?;
        let resp_body = response.text().await?;
        let pseudonyms =
            SoapEnvelope::<GetPseudonymsForResponseBody>::try_from(resp_body.as_str())?;

        Ok((
            domain,
            pseudonyms.body.get_pseudonyms_for_response.returns.psn,
        ))
    }

    async fn pseudonymize_secondary(
        &self,
        trial: &str,
        lab: &str,
        mpi: String,
        count: String,
    ) -> anyhow::Result<Vec<String>> {
        let body = gpas::create_secondary_psn_request(format!("{trial}_{lab}"), mpi, count)?;
        let request = self
            .client
            .post(
                format!(
                    "{}/ttp-fhir/fhir/gpas/$pseudonymize-secondary",
                    self.gpas.base_url
                )
                .as_str(),
            )
            .body(serde_json::to_string(&body)?);

        let response = request.send().await?;
        let body = response.text().await?;
        let params = serde_json::from_str(body.as_str())?;
        Ok(gpas::parse_secondary(params))
    }

    pub(crate) async fn get_secondary_domains(&self, trial: String) -> anyhow::Result<Vec<String>> {
        // get trial domain
        let body: String = gpas::create_get_domain_request(trial).try_into()?;
        let request = self
            .client
            .post(format!("{}/gpas/DomainService?wsdl", self.gpas.base_url).as_str())
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/soap+xml"),
            )
            .body(body);

        let response = request.send().await?;
        let resp_body = response.text().await?;
        let matched = SoapEnvelope::<GetDomainResponseBody>::try_from(resp_body.as_str())?;

        Ok(matched
            .body
            .get_domain_response
            .domain
            .child_domain_names
            .unwrap_or_default())
    }

    async fn send_epix(&self, body: String) -> Result<Response, Error> {
        let request = self
            .client
            .post(format!("{}/epix/epixService?wsdl", self.epix.base_url).as_str())
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/soap+xml"),
            )
            .body(body);

        request.send().await
    }
}

// async fn parse_response<'a, T: Deserialize>(response: Response) -> anyhow::Result<T>
// where
//     SoapEnvelope<T>: TryFrom<&'a str>, // where
//                                        //     SoapEnvelope<T>: TryFrom<&'a str>,
// {
//     if !response.status().is_success() {
//         let resp_text = response.text().await?;
//         let fault = FaultEnvelope::try_from(resp_text);
//         Err(anyhow!(
//             fault.map(|f: FaultEnvelope| f.body.fault.faultstring)?
//         ))
//     } else {
//         let resp_text = response.text().await?.as_str();
//         let soap: SoapEnvelope<T> = SoapEnvelope::<T>::try_from(resp_text)?;
//
//         Ok(soap.body)
//     }
// }

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename = "soap:Envelope")]
pub(crate) struct FaultEnvelope {
    #[serde(rename = "soap:Body")]
    pub(crate) body: FaultBody,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct FaultBody {
    #[serde(rename = "soap:Fault")]
    pub(crate) fault: Fault,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct Fault {
    pub(crate) faultcode: String,
    pub(crate) faultstring: String,
    pub(crate) detail: FaultException,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InvalidParameterException {
    parameter_name: String,
    error_code: (),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) enum FaultException {
    #[serde(rename = "ns1:DomainInUseException")]
    DomainInUse(()),
    #[serde(rename = "ns1:DuplicateEntryException")]
    DuplicateEntry(()),
    #[serde(rename = "ns1:InvalidParameterException")]
    InvalidParameter(InvalidParameterException),
}

impl TryFrom<String> for FaultEnvelope {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let config = serde_xml_rs::SerdeXml::new()
            .namespace("ns1", "http://service.epix.ttp.icmvc.emau.org/")
            .namespace("ns2", "http://psn.ttp.ganimed.icmvc.emau.org/")
            .namespace("soap", "http://schemas.xmlsoap.org/soap/envelope/");

        let env: Self = config.from_str(value.as_str())?;
        Ok(env)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename = "soap:Envelope")]
pub(crate) struct SoapEnvelope<T> {
    #[serde(rename = "soap:Body")]
    pub(crate) body: T,
}

impl<T> SoapEnvelope<T> {
    pub(super) fn new(body: T) -> Self {
        SoapEnvelope::<T> { body }
    }
}

impl<'a, T: serde::Deserialize<'a>> TryFrom<&str> for SoapEnvelope<T> {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let env: Self = serde_xml_rs::from_str(value)?;
        Ok(env)
    }
}

impl<T: serde::Serialize> TryInto<String> for SoapEnvelope<T> {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<String, Self::Error> {
        let config = serde_xml_rs::SerdeXml::new()
            .namespace("ns1", "http://service.epix.ttp.icmvc.emau.org/")
            .namespace("ns2", "http://psn.ttp.ganimed.icmvc.emau.org/")
            .namespace("ns2", "http://service.epix.ttp.icmvc.emau.org/")
            .namespace("soap", "http://schemas.xmlsoap.org/soap/envelope/");

        let env: String = config.to_string(&self)?;
        Ok(env)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::config::{AppConfig, Epix, Gpas, Ttp};
    use crate::ttp::client::TtpClient;
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use reqwest::header::CONTENT_TYPE;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    pub(crate) fn setup_config(base_url: String) -> AppConfig {
        AppConfig {
            ttp: Ttp {
                epix: Epix {
                    base_url: base_url.clone(),
                    domain: Default::default(),
                    identifier_domain: Default::default(),
                    data_source: Default::default(),
                },
                gpas: Gpas { base_url },
                timeout: 5,
            },
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_connection_ok() {
        use httpmock::prelude::*;

        init();
        let server = MockServer::start();
        let epix_metadata = server.mock(|when, then| {
            when.method(GET)
                .path("/ttp-fhir/fhir/epix/metadata")
                .header_exists(CONTENT_TYPE.as_str());
            then.status(200).body("OK");
        });
        let gpas_metadata = server.mock(|when, then| {
            when.method(GET)
                .path("/ttp-fhir/fhir/gpas/metadata")
                .header_exists(CONTENT_TYPE.as_str());
            then.status(200).body("OK");
        });

        let config = setup_config(server.base_url());
        // create new client
        let client = TtpClient::new(&config.ttp).await;

        // connection test
        let test_result = client.unwrap().test_connection().await;

        // mocks were called once
        epix_metadata.assert();
        gpas_metadata.assert();

        // assert client is created and initialized
        assert!(test_result.is_ok());
    }

    #[tokio::test]
    async fn test_connection_error() {
        use httpmock::prelude::*;

        let server = MockServer::start();
        let metadata_mock = server.mock(|when, then| {
            when.method(GET)
                .path("/ttp-fhir/fhir/epix/metadata")
                .header_exists(CONTENT_TYPE.as_str());
            then.status(404);
        });

        let config = setup_config(server.base_url());
        // create new client
        let client = TtpClient::new(&config.ttp).await;

        // connection test
        let test_result = client.unwrap().test_connection().await;

        // mock was called once
        metadata_mock.assert();

        // assert client connection test failed
        assert!(test_result.is_err());
    }

    #[tokio::test]
    async fn test_get_possible_matches_for_person_response() {
        let test_response = r#"<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
    <soap:Body>
        <ns1:getPossibleMatchesForPersonResponse xmlns:ns1="http://service.epix.ttp.icmvc.emau.org/">
            <return>
                <creationType>AUTOMATIC</creationType>
                <linkId>62</linkId>
                <possibleMatchCreated>2025-11-21T14:16:23.242+01:00</possibleMatchCreated>
                <priority>OPEN</priority>
                <probability>3.1481482315455955</probability>
                <assignedIdentity>
                    <birthDate>1972-01-01T00:00:00+01:00</birthDate>
                    <birthPlace>Musterstadt</birthPlace>
                    <firstName>Max</firstName>
                    <lastName>Muster</lastName>
                    <deactivated>false</deactivated>
                    <identityCreated>2025-11-21T14:16:23.242+01:00</identityCreated>
                    <identityId>53</identityId>
                    <identityLastEdited>2025-11-21T14:16:23.242+01:00</identityLastEdited>
                    <identityVersion>1</identityVersion>
                    <personId>53</personId>
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
                        <contactId>53</contactId>
                        <contactLastEdited>2025-11-21T14:16:23.242+01:00</contactLastEdited>
                        <contactVersion>1</contactVersion>
                        <deactivated>false</deactivated>
                        <identityId>53</identityId>
                    </contacts>
                </assignedIdentity>
                <matchingMPIIdentity>
                    <identity>
                        <birthDate>1972-01-01T00:00:00+01:00</birthDate>
                        <birthPlace>Musterstadt</birthPlace>
                        <firstName>Max</firstName>
                        <lastName>Mustermann</lastName>
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
                        <value>1001000000011</value>
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
                    <value>1001000000073</value>
                </requestedMPI>
            </return>
        </ns1:getPossibleMatchesForPersonResponse>
    </soap:Body>
</soap:Envelope>"#;

        let server = MockServer::start();
        let epix_soap_mock = server.mock(|when, then| {
            when.method(POST).path("/epix/epixService");
            then.status(200).body(test_response);
        });

        let config = setup_config(server.base_url());
        // create new client
        let client = TtpClient::new(&config.ttp).await;

        // check duplicates
        let test_result = client
            .unwrap()
            .possible_matches_for_person("test".to_string())
            .await;

        // mocks were called once
        epix_soap_mock.assert();

        // assert client is created and initialized
        assert!(test_result.is_ok());
    }
}
