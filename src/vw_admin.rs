extern crate reqwest;
extern crate serde;
extern crate thiserror;

use reqwest::Response;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::time::{Duration, Instant};
use thiserror::Error;

const COOKIE_LIFESPAN: Duration = Duration::from_secs(20 * 60);

#[derive(Error, Debug)]
pub enum ResponseError {
    #[error("vaultwarden error {0}")]
    ApiError(String),

    #[error("http error making request {0:?}")]
    HttpError(#[from] reqwest::Error),
}

#[derive(Debug, Deserialize)]
pub struct User {
    #[serde(rename = "Email")]
    email: String,
    #[serde(rename = "_Status")]
    status: i32,
}

impl User {
    pub fn get_email(&self) -> String {
        self.email.clone()
    }

    pub fn is_disabled(&self) -> bool {
        // HACK: Magic number
        self.status == 2
    }
}

pub struct Client {
    url: String,
    admin_token: String,
    root_cert_file: String,
    cookie: Option<String>,
    cookie_created: Option<Instant>,
}

impl Client {
    /// Create new instance of client
    pub fn new(url: String, admin_token: String, root_cert_file: String) -> Client {
        Client {
            url,
            admin_token,
            root_cert_file,
            cookie: None,
            cookie_created: None,
        }
    }

    fn get_root_cert(&self) -> reqwest::Certificate {
        let mut buf = Vec::new();

        // read a local binary DER encoded certificate
        File::open(&self.root_cert_file)
            .expect("Could not open root cert file")
            .read_to_end(&mut buf)
            .expect("Could not read root cert file");

        reqwest::Certificate::from_der(&buf).expect("Could not load DER root cert file")
    }

    fn get_http_client(&self) -> reqwest::Client {
        let mut client = reqwest::Client::builder().redirect(reqwest::RedirectPolicy::none());

        if !&self.root_cert_file.is_empty() {
            let cert = self.get_root_cert();
            client = client.add_root_certificate(cert);
        }

        client.build().expect("Failed to build http client")
    }

    /// Authenticate client
    fn auth(&mut self) -> Result<Response, ResponseError> {
        let cookie_created = Instant::now();
        let client = self.get_http_client();
        let result = client
            .post(format!("{}{}", &self.url, "/admin/").as_str())
            .form(&[("token", &self.admin_token)])
            .send()?
            .error_for_status()?;

        let cookie = result
            .headers()
            .get(reqwest::header::SET_COOKIE)
            .ok_or_else(|| {
                ResponseError::ApiError(String::from("Could not read authentication cookie"))
            })?;

        self.cookie = cookie.to_str().map(String::from).ok();
        self.cookie_created = Some(cookie_created);

        Ok(result)
    }

    fn cookie_expired(&self) -> bool {
        match &self.cookie {
            Some(_) => self
                .cookie_created
                .map_or(true, |created| (created.elapsed() >= COOKIE_LIFESPAN)),
            None => true,
        }
    }

    /// Ensure that the client has a current auth cookie
    fn ensure_auth(&mut self) -> Result<(), ResponseError> {
        if self.cookie_expired() {
            match self.auth() {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            }?
        }

        Ok(())
    }

    /// Make an authenticated GET to Bitwarden Admin
    fn get(&mut self, path: &str) -> Result<Response, ResponseError> {
        self.ensure_auth()?;

        let url = format!("{}/admin{}", &self.url, path);
        let client = self.get_http_client();
        let request = client.get(url.as_str()).header(
            reqwest::header::COOKIE,
            self.cookie
                .as_ref()
                .expect("No cookie found to add to header")
                .clone(),
        );

        let response = request.send()?.error_for_status()?;

        Ok(response)
    }

    /// Make authenticated POST to Bitwarden Admin with JSON data
    fn post(
        &mut self,
        path: &str,
        json: &HashMap<String, String>,
    ) -> Result<Response, ResponseError> {
        self.ensure_auth()?;

        let url = format!("{}/admin{}", &self.url, path);
        let client = self.get_http_client();
        let request = client.post(url.as_str()).json(&json).header(
            reqwest::header::COOKIE,
            self.cookie
                .as_ref()
                .expect("No cookie found to add to header")
                .clone(),
        );

        let response = request.send()?.error_for_status()?;

        Ok(response)
    }

    /// Invite user with provided email
    pub fn invite(&mut self, email: &str) -> Result<Response, ResponseError> {
        let mut json = HashMap::new();
        json.insert("email".to_string(), email.to_string());

        self.post("/invite", &json)
    }

    /// Get all existing users
    pub fn users(&mut self) -> Result<Vec<User>, ResponseError> {
        let all_users: Vec<User> = self.get("/users")?.json()?;
        Ok(all_users)
    }
}
