use crate::api::IdRequest;
use crate::config::Ttp;
use anyhow::anyhow;
use fhir_model::r4b::resources::{Parameters, ParametersParameter, ParametersParameterValue};
use fhir_model::r4b::types::Coding;
use log::{error, info};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use reqwest::{header, Client};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use std::time::Duration;

#[derive(Debug, Clone)]
pub(crate) struct FhirClient {
    client: ClientWithMiddleware,
    epix_base: String,
    gpas_base: String,
}

impl FhirClient {
    pub(crate) async fn new(config: &Ttp) -> Result<Self, anyhow::Error> {
        // default headers
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/fhir+json"),
        );
        // set auth header as default
        if let Some(auth) = config.auth.clone().and_then(|a| a.basic) {
            // auth header
            let auth_value =
                create_auth_header(auth.username.as_str(), Some(auth.password.as_str()));
            headers.insert(header::AUTHORIZATION, auth_value);
        };

        // retry
        let retry = ExponentialBackoff::builder()
            .retry_bounds(
                Duration::from_secs(config.retry.wait),
                Duration::from_secs(config.retry.max_wait),
            )
            .build_with_max_retries(config.retry.count);

        // client with retry middleware
        let client = ClientBuilder::new(
            Client::builder()
                .default_headers(headers.clone())
                .timeout(Duration::from_secs(config.retry.timeout))
                .build()?,
        )
        .with(RetryTransientMiddleware::new_with_policy(retry))
        .build();

        Ok(FhirClient {
            client,
            epix_base: config.epix.base_url.clone(),
            gpas_base: config.gpas.base_url.clone(),
        })
    }

    pub(crate) async fn test_connection(&self) -> anyhow::Result<()> {
        // test epix
        self.get_metadata(format!("{}/ttp-fhir/fhir/epix", self.epix_base).as_str())
            .await?;
        info!("Connection test to E-PIX successful");

        // test gpas
        self.get_metadata(format!("{}/ttp-fhir/fhir/gpas", self.gpas_base).as_str())
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
                        .value(ParametersParameterValue::String("kks".to_string()))
                        .build()?,
                ),
                Some(
                    ParametersParameter::builder()
                        .name("source".to_string())
                        .value(ParametersParameterValue::String("socramate".to_string()))
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
                        .name("saveAction".to_string())
                        .value(ParametersParameterValue::Coding(
                            Coding::builder()
                                .system(
                                    "https://ths-greifswald.de/fhir/CodeSystem/epix/SaveAction"
                                        .to_string(),
                                )
                                .code("DONT_SAVE_ON_PERFECT_MATCH_EXCEPT_CONTACTS".to_string())
                                .build()?,
                        ))
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
            .post(format!("{}/$addPatient", self.epix_base.as_str()).as_str())
            .body(serde_json::to_string(&body)?);

        let response = request.send().await?;

        Ok(serde_json::from_str(response.text().await?.as_str())?)
    }

    pub(crate) async fn pseudonymize(
        &self,
        mpi: String,
        request: IdRequest,
    ) -> Result<Parameters, anyhow::Error> {
        let body = Parameters::builder()
            .parameter(vec![
                Some(
                    ParametersParameter::builder()
                        .name("target".to_string())
                        .value(ParametersParameterValue::String(request.study))
                        .build()?,
                ),
                Some(
                    ParametersParameter::builder()
                        .name("original".to_string())
                        .value(ParametersParameterValue::String(mpi))
                        .build()?,
                ),
            ])
            .build()?;

        let request = self
            .client
            // todo allow multi psn
            .post(
                format!(
                    "{}/ttp-fhir/fhir/$pseudonymizeAllowCreate",
                    self.gpas_base.as_str()
                )
                .as_str(),
            )
            .body(serde_json::to_string(&body)?);

        let response = request.send().await?;

        Ok(serde_json::from_str(response.text().await?.as_str())?)
    }

    pub(crate) async fn add_domain(&self, domain: &str) -> Result<Parameters, anyhow::Error> {
        let soap = r#"<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/" xmlns:psn="http://psn.ttp.ganimed.icmvc.emau.org/">
                           <soap:Header></soap:Header>
                           <soap:Body>
                             <psn:addDomain>
                               <domainDTO>
                                 <name>test</name>
                                 <label>test</label>
                                 <checkDigitClass>org.emau.icmvc.ganimed.ttp.psn.generator.NoCheckDigits</checkDigitClass>
                                 <alphabet>org.emau.icmvc.ganimed.ttp.psn.alphabets.Symbol32</alphabet>
                                 <parentDomainNames>test</parentDomainNames>
                                 <config>
                                   <psnLength>16</psnLength>
                                   <psnPrefix>PSN</psnPrefix>
                                   <psnsDeletable>false</psnsDeletable>
                                 </config>
                               </domainDTO>
                             </psn:addDomain>
                           </soap:Body>
                         </soap:Envelope>"#;

        let request = self
            .client
            // todo allow multi psn
            .post(format!("{}/$pseudonymizeAllowCreate", self.gpas_base.as_str()).as_str())
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/fhir+xml"),
            )
            .body(soap);

        let response = request.send().await?;

        Ok(serde_json::from_str(response.text().await?.as_str())?)
    }
}

fn create_auth_header(user: &str, password: Option<&str>) -> HeaderValue {
    let builder = Client::new()
        .get("http://localhost")
        .basic_auth(user, password);

    builder
        .build()
        .unwrap()
        .headers()
        .get(AUTHORIZATION)
        .unwrap()
        .clone()
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::client::FhirClient;
    use crate::config::{AppConfig, Auth, BasicAuth, Epix, Gpas, Retry, Ttp};

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    pub(crate) fn setup_config(base_url: String) -> AppConfig {
        AppConfig {
            ttp: Ttp {
                epix: Epix {
                    base_url: base_url.clone(),
                },
                gpas: Gpas { base_url },
                auth: Some(Auth {
                    basic: Some(BasicAuth {
                        username: "foo".to_string(),
                        password: "bar".to_string(),
                    }),
                }),
                retry: Retry {
                    timeout: 5,
                    ..Default::default()
                },
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
                .header_exists("Authorization");
            then.status(200).body("OK");
        });
        let gpas_metadata = server.mock(|when, then| {
            when.method(GET)
                .path("/ttp-fhir/fhir/gpas/metadata")
                .header_exists("Authorization");
            then.status(200).body("OK");
        });

        let config = setup_config(server.base_url());
        // create new client
        let client = FhirClient::new(&config.ttp).await;

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
                .header_exists("Authorization");
            then.status(404);
        });

        let config = setup_config(server.base_url());
        // create new client
        let client = FhirClient::new(&config.ttp).await;

        // connection test
        let test_result = client.unwrap().test_connection().await;

        // mock was called once
        metadata_mock.assert();

        // assert client connection test failed
        assert!(test_result.is_err());
    }
}
