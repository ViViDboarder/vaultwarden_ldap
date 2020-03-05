extern crate reqwest;
extern crate serde;

use reqwest::Response;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::time::{Duration, Instant};

const COOKIE_LIFESPAN: Duration = Duration::from_secs(20 * 60);

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
    cookie: Option<String>,
    cookie_created: Option<Instant>,
}

impl Client {
    /// Create new instance of client
    pub fn new(url: String, admin_token: String) -> Client {
        Client {
            url,
            admin_token,
            cookie: None,
            cookie_created: None,
        }
    }

    /// Authenticate client
    fn auth(&mut self) -> Response {
        let cookie_created = Instant::now();
        let client = reqwest::Client::builder()
            // Avoid redirects because server will redirect to admin page after auth
            .redirect(reqwest::RedirectPolicy::none())
            .build()
            .unwrap();
        let result = client
            .post(format!("{}{}", &self.url, "/admin/").as_str())
            .form(&[("token", &self.admin_token)])
            .send()
            .unwrap_or_else(|e| {
                panic!("Could not authenticate with {}. {:?}", &self.url, e);
            });

        // TODO: Handle error statuses

        if let Some(cookie) = result.headers().get(reqwest::header::SET_COOKIE) {
            self.cookie = cookie.to_str().map(|s| String::from(s)).ok();
            self.cookie_created = Some(cookie_created);
        } else {
            panic!("Could not authenticate.")
        }

        result
    }

    /// Ensure that the client has a current auth cookie
    fn ensure_auth(&mut self) {
        match &self.cookie {
            Some(_) => {
                if self
                    .cookie_created
                    .map_or(true, |created| (created.elapsed() >= COOKIE_LIFESPAN))
                {
                    self.auth();
                }
            }
            None => {
                self.auth();
            }
        };
        // TODO: handle errors
    }

    /// Make an authenticated GET to Bitwarden Admin
    fn get(&mut self, path: &str) -> Response {
        self.ensure_auth();

        match &self.cookie {
            None => {
                panic!("We haven't authenticated. Must be an error");
            }
            Some(cookie) => {
                let url = format!("{}/admin{}", &self.url, path);
                let request = reqwest::Client::new()
                    .get(url.as_str())
                    .header(reqwest::header::COOKIE, cookie.clone());
                let response = request.send().unwrap_or_else(|e| {
                    panic!("Could not call with {}. {:?}", url, e);
                });

                // TODO: Handle error statuses

                return response;
            }
        }
    }

    /// Make authenticated POST to Bitwarden Admin with JSON data
    fn post(&mut self, path: &str, json: &HashMap<String, String>) -> Response {
        self.ensure_auth();

        match &self.cookie {
            None => {
                panic!("We haven't authenticated. Must be an error");
            }
            Some(cookie) => {
                let url = format!("{}/admin{}", &self.url, path);
                let request = reqwest::Client::new()
                    .post(url.as_str())
                    .header("Cookie", cookie.clone())
                    .json(&json);
                let response = request.send().unwrap_or_else(|e| {
                    panic!("Could not call with {}. {:?}", url, e);
                });

                // TODO: Handle error statuses

                return response;
            }
        }
    }

    /// Invite user with provided email
    pub fn invite(&mut self, email: &str) -> Response {
        let mut json = HashMap::new();
        json.insert("email".to_string(), email.to_string());

        self.post("/invite", &json)
    }

    /// Get all existing users
    pub fn users(&mut self) -> Result<Vec<User>, Box<dyn Error>> {
        let all_users: Vec<User> = self.get("/users").json()?;
        Ok(all_users)
    }
}
