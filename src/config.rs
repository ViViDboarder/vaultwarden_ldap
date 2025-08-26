extern crate serde;

use std::env;
use std::fs;

use serde::Deserialize;

pub type Pass = String;

const CONFIG_PATH_DEFAULT: &str = "config.toml";

/// Returns config path from envioronment or a provided default value
pub fn get_config_path() -> String {
    match env::var("CONFIG_PATH") {
        Ok(config_path) => config_path,
        Err(_) => String::from(CONFIG_PATH_DEFAULT),
    }
}

// Tries to read configuration from file, failing that from the environment,
// panics if it can't
pub fn read_config() -> Config {
    match read_config_from_file() {
        Ok(config) => config,
        Err(err) => {
            println!("Error: {err}");
            match read_config_from_env() {
                Ok(config) => config,
                Err(err) => panic!("{}", err),
            }
        }
    }
}

/// Tries to read configuration from file
pub fn read_config_from_file() -> Result<Config, String> {
    let config_path = get_config_path();

    let contents = fs::read_to_string(&config_path)
        .map_err(|err| format!("Error: Failed to open config file at {config_path}: {err}"))?;
    let config: Config = toml::from_str(contents.as_str())
        .map_err(|err| format!("Error: Failed to parse config file at {config_path}: {err}"))?;

    println!("Config read from file at {config_path}");
    Ok(config)
}

// Tries to read configuration from environment
pub fn read_config_from_env() -> Result<Config, String> {
    let config = envy::prefixed("APP_")
        .from_env()
        .map_err(|err| format!("Error: Could not parse config from env: {err}"))?;

    println!("Config read from environment");
    Ok(config)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
/// Contains all config values for LDAP syncing
pub struct Config {
    // Bitwarden connection config
    vaultwarden_url: String,
    vaultwarden_admin_token: Option<String>,
    vaultwarden_admin_token_file: Option<String>,
    vaultwarden_root_cert_file: Option<String>,
    // LDAP Connection config
    ldap_host: String,
    ldap_scheme: Option<String>,
    ldap_ssl: Option<bool>,
    ldap_starttls: Option<bool>,
    ldap_port: Option<u16>,
    ldap_no_tls_verify: Option<bool>,
    // LDAP auth config
    ldap_bind_dn: Option<String>,
    ldap_bind_password: Option<Pass>,
    ldap_bind_password_file: Option<Pass>,
    // LDAP search config
    ldap_search_base_dn: String,
    ldap_search_filter: String,
    // LDAP record attributes
    ldap_mail_field: Option<String>,
    // Interval syncing config
    ldap_sync_interval_seconds: Option<u64>,
    // Should start background sync loop
    ldap_sync_loop: Option<bool>,
}

impl Config {
    /// Create a config instance from file
    pub fn from_file() -> Config {
        read_config()
    }

    pub fn get_vaultwarden_url(&self) -> String {
        self.vaultwarden_url.clone()
    }

    pub fn get_vaultwarden_admin_token(&self) -> String {
        match &self.vaultwarden_admin_token {
            // If we have a token value, return it
            Some(vaultwarden_admin_token) => vaultwarden_admin_token.clone(),
            // If not, try to read it from a file
            None => match &self.vaultwarden_admin_token_file {
                Some(vaultwarden_admin_token_file) => {
                    fs::read_to_string(vaultwarden_admin_token_file)
                        .expect("Failed to read vaultwarden admin token file")
                        .trim()
                        .to_string()
                }
                // If we don't have a token value or a file, raise an error
                None => panic!("No vaultwarden admin token provided"),
            },
        }
    }

    pub fn get_vaultwarden_root_cert_file(&self) -> String {
        match &self.vaultwarden_root_cert_file {
            Some(vaultwarden_root_cert_file) => vaultwarden_root_cert_file.clone(),
            None => String::new(),
        }
    }

    pub fn get_ldap_url(&self) -> String {
        format!(
            "{}://{}:{}",
            self.get_ldap_scheme(),
            self.get_ldap_host(),
            self.get_ldap_port()
        )
    }

    pub fn get_ldap_host(&self) -> String {
        self.ldap_host.clone()
    }

    pub fn get_ldap_scheme(&self) -> String {
        match &self.ldap_scheme {
            Some(ldap_scheme) => ldap_scheme.clone(),
            None => {
                if self.get_ldap_ssl() {
                    String::from("ldaps")
                } else {
                    String::from("ldap")
                }
            }
        }
    }

    pub fn get_ldap_ssl(&self) -> bool {
        self.ldap_ssl.unwrap_or(false)
    }

    pub fn get_ldap_starttls(&self) -> bool {
        self.ldap_starttls.unwrap_or(false)
    }

    pub fn get_ldap_no_tls_verify(&self) -> bool {
        self.ldap_no_tls_verify.unwrap_or(false)
    }

    pub fn get_ldap_port(&self) -> u16 {
        match self.ldap_port {
            Some(ldap_port) => ldap_port,
            None => {
                if self.get_ldap_ssl() {
                    636
                } else {
                    389
                }
            }
        }
    }

    pub fn get_ldap_bind_dn(&self) -> Option<String> {
        self.ldap_bind_dn.clone()
    }

    pub fn get_ldap_bind_password(&self) -> Option<String> {
        match &self.ldap_bind_password {
            Some(ldap_bind_password) => Some(ldap_bind_password.clone()),
            None => self
                .ldap_bind_password_file
                .as_ref()
                .map(|ldap_bind_password_file| {
                    fs::read_to_string(ldap_bind_password_file)
                        .expect("Failed to read ldap bind password file")
                        .trim()
                        .to_string()
                }),
        }
    }

    pub fn get_ldap_search_base_dn(&self) -> String {
        self.ldap_search_base_dn.clone()
    }

    pub fn get_ldap_search_filter(&self) -> String {
        self.ldap_search_filter.clone()
    }

    pub fn get_ldap_mail_field(&self) -> String {
        match &self.ldap_mail_field {
            Some(mail_field) => mail_field.clone(),
            None => String::from("mail"),
        }
    }

    pub fn get_ldap_sync_interval_seconds(&self) -> u64 {
        self.ldap_sync_interval_seconds.unwrap_or(60)
    }

    pub fn get_ldap_sync_loop(&self) -> bool {
        self.ldap_sync_loop.unwrap_or(true)
    }
}
