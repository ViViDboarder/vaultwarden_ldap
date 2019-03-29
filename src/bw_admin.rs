extern crate reqwest;
extern crate serde_json;

use reqwest::Response;
use std::collections::HashMap;
use std::time::{Duration, Instant};

const COOKIE_LIFESPAN: Duration = Duration::from_secs(20 * 60);

pub struct Client {
    url: String,
    admin_token: String,
    cookie: Option<String>,
    cookie_created: Option<Instant>,
}

impl Client {
    pub fn new(url: String, admin_token: String) -> Client {
        Client {
            url,
            admin_token,
            cookie: None,
            cookie_created: None,
        }
    }

    fn auth(&mut self) -> Response {
        let cookie_created = Instant::now();
        let result = reqwest::Client::new()
            .post(format!("{}{}", &self.url, "/admin/").as_str())
            .form(&[("token", &self.admin_token)])
            .send()
            .unwrap_or_else(|e| {
                panic!("Could not authenticate with {}. {:?}", &self.url, e);
            });

        // TODO: Handle error statuses

        println!("Auth headers: {:?}", result.headers());

        if let Some(cookie) = result.headers().get(reqwest::header::SET_COOKIE) {
            self.cookie = cookie.to_str().map(|s| String::from(s)).ok();
            self.cookie_created = Some(cookie_created);
        } else {
            panic!("No cookie to set!")
        }

        result
    }

    fn ensure_auth(&mut self) {
        match &self.cookie {
            Some(_) => {
                if self
                    .cookie_created
                    .map_or(true, |created| (created.elapsed() >= COOKIE_LIFESPAN))
                {
                    let response = self.auth();
                    println!("Auth response: {:?}", response);
                }
            }
            None => {
                let response = self.auth();
                println!("Auth response: {:?}", response);
            }
        };
        // TODO: handle errors
    }

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

    pub fn invite(&mut self, email: &str) -> Response {
        let mut json = HashMap::new();
        json.insert("email".to_string(), email.to_string());

        self.post("/invite", &json)
    }
}
