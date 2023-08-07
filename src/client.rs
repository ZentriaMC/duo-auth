use std::{future::Future, sync::Arc, time::Duration};

use reqwest::{Client, Method, Request, Url};
use serde::{de::DeserializeOwned, Deserialize};

use super::{
    errors::Error,
    request::{DuoRequest, Parameters},
    response::DuoResponse,
    types::PreauthResponse,
    types::{
        AuthRequest, AuthStatusResponse, EnrollResponse, EnrollStatusResponse, PreauthRequest,
    },
    StdError,
};

pub struct DuoClient(Arc<DuoClientInner>);

struct DuoClientInner {
    base_url: Url,
    ikey: String,
    skey: String,

    client: reqwest::Client,
}

impl DuoClient {
    pub fn new<D, I, S>(api_domain: D, ikey: I, skey: S) -> Result<DuoClient, Error>
    where
        D: Into<String>,
        I: Into<String>,
        S: Into<String>,
    {
        let client = reqwest::Client::builder()
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .map_err(Error::unspecified)?;

        Self::new_with_client(client, api_domain, ikey, skey)
    }

    pub fn new_with_client<C, D, I, S>(
        client: C,
        api_domain: D,
        ikey: I,
        skey: S,
    ) -> Result<DuoClient, Error>
    where
        C: Into<Client>,
        D: Into<String>,
        I: Into<String>,
        S: Into<String>,
    {
        let api_domain = api_domain.into();

        let base_url = match Url::parse(&api_domain) {
            Ok(url) => url,
            Err(err) => {
                return Err(Error::InvalidApiDomain {
                    domain: api_domain,
                    cause: err.into(),
                })
            }
        };

        // Fail fast when there's no domain
        let _ = base_url
            .host_str()
            .ok_or_else(|| Error::InvalidApiDomain {
                domain: api_domain,
                cause: "no domain in url".into(),
            })?
            .to_string();

        Ok(DuoClient(Arc::new(DuoClientInner {
            base_url,
            ikey: ikey.into(),
            skey: skey.into(),
            client: client.into(),
        })))
    }

    pub fn auth(&self, data: AuthRequest) -> impl Future<Output = Result<String, Error>> {
        let this = Arc::clone(&self.0);

        async move { Self::request_auth(this, data).await }
    }

    pub fn auth_status<S: Into<String>>(
        &self,
        tx_id: S,
    ) -> impl Future<Output = Result<AuthStatusResponse, Error>> {
        let this = Arc::clone(&self.0);

        async move {
            let txid: String = tx_id.into();
            Self::request_auth_status(this, &txid).await
        }
    }

    pub fn auth_wait(&self, data: AuthRequest) -> impl Future<Output = Result<bool, StdError>> {
        let this = Arc::clone(&self.0);

        async move {
            let txid = Self::request_auth(this.clone(), data).await?;
            let mut status: Option<bool>;

            loop {
                status = Self::request_auth_status(this.clone(), &txid)
                    .await?
                    .ready();
                match status {
                    None => tokio::time::sleep(Duration::from_secs(2)).await,
                    Some(v) => return Ok(v),
                }
            }
        }
    }

    pub fn check(&self) -> impl Future<Output = Result<u64, Error>> {
        let this = Arc::clone(&self.0);

        async move {
            #[derive(Deserialize, Debug)]
            struct CheckResponse {
                time: u64,
            }

            let request =
                Self::new_request(&this, Method::GET, "/auth/v2/check", Parameters::default())?;
            Self::send_request_json::<CheckResponse>(&this.client, request)
                .await
                .map(|r| r.time)
        }
    }

    pub fn enroll<U: Into<String>>(
        &self,
        username: Option<U>,
        valid_secs: Option<u64>,
    ) -> impl Future<Output = Result<EnrollResponse, Error>> {
        let this = Arc::clone(&self.0);

        async move { Self::request_enroll(this, username, valid_secs).await }
    }

    pub fn enroll_status<U: Into<String>, A: Into<String>>(
        &self,
        user_id: U,
        activation_code: A,
    ) -> impl Future<Output = Result<EnrollStatusResponse, Error>> {
        let this = Arc::clone(&self.0);

        async move { Self::request_enroll_status(this, user_id, activation_code).await }
    }

    pub fn ping(&self) -> impl Future<Output = Result<u64, Error>> {
        let this = Arc::clone(&self.0);

        async move {
            #[derive(Deserialize, Debug)]
            struct PingResponse {
                time: u64,
            }

            let request =
                Self::new_request(&this, Method::GET, "/auth/v2/ping", Parameters::default())?;
            Self::send_request_json::<PingResponse>(&this.client, request)
                .await
                .map(|r| r.time)
        }
    }

    pub fn preauth(
        &self,
        data: PreauthRequest,
    ) -> impl Future<Output = Result<PreauthResponse, Error>> {
        let this = Arc::clone(&self.0);

        async move { Self::request_preauth(this, data).await }
    }

    async fn request_auth(this: Arc<DuoClientInner>, data: AuthRequest) -> Result<String, Error> {
        let mut parameters = Parameters::default();
        parameters.set("async", "1");
        data.apply(&mut parameters);

        #[derive(Deserialize, Debug)]
        struct AuthResponse {
            txid: String,
        }

        let request = Self::new_request(&this, Method::POST, "/auth/v2/auth", parameters)?;
        Self::send_request_json::<AuthResponse>(&this.client, request)
            .await
            .map(|r| r.txid)
    }

    async fn request_auth_status(
        this: Arc<DuoClientInner>,
        tx_id: &str,
    ) -> Result<AuthStatusResponse, Error> {
        let mut parameters = Parameters::default();
        parameters.set("txid", tx_id);

        let request = Self::new_request(&this, Method::GET, "/auth/v2/auth_status", parameters)?;
        Self::send_request_json(&this.client, request).await
    }

    async fn request_enroll<U: Into<String>>(
        this: Arc<DuoClientInner>,
        username: Option<U>,
        valid_secs: Option<u64>,
    ) -> Result<EnrollResponse, Error> {
        let mut parameters = Parameters::default();
        parameters.set_opt("username", username);
        parameters.set_opt("valid_secs", valid_secs.map(|v| v.to_string()));

        let request = Self::new_request(&this, Method::POST, "/auth/v2/enroll", parameters)?;
        Self::send_request_json(&this.client, request).await
    }

    async fn request_enroll_status<U: Into<String>, A: Into<String>>(
        this: Arc<DuoClientInner>,
        user_id: U,
        activation_code: A,
    ) -> Result<EnrollStatusResponse, Error> {
        let mut parameters = Parameters::default();
        parameters.set("user_id", user_id);
        parameters.set("activation_code", activation_code);

        let request = Self::new_request(&this, Method::POST, "/auth/v2/enroll_status", parameters)?;
        Self::send_request_json(&this.client, request).await
    }

    async fn request_preauth(
        this: Arc<DuoClientInner>,
        data: PreauthRequest,
    ) -> Result<PreauthResponse, Error> {
        let mut parameters = Parameters::default();
        data.apply(&mut parameters);

        let request = Self::new_request(&this, Method::POST, "/auth/v2/preauth", parameters)?;
        Self::send_request_json(&this.client, request).await
    }

    fn new_request<P: Into<String>>(
        this: &Arc<DuoClientInner>,
        method: Method,
        path: P,
        parameters: Parameters,
    ) -> Result<Request, Error> {
        DuoRequest::new(this.base_url.clone(), method, path, parameters)
            .build(&this.client, &this.ikey, &this.skey)
            .map_err(Error::unspecified)
    }

    async fn send_request_json<T>(client: &Client, request: Request) -> Result<T, Error>
    where
        T: DeserializeOwned + std::fmt::Debug,
    {
        let response = client.execute(request).await.map_err(Error::unspecified)?;

        let body = response
            .json::<DuoResponse<T>>()
            .await
            .map_err(Error::unspecified)?;

        body.ok()
    }
}
