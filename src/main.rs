extern crate ldap3;

use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

use ldap3::{DerefAliases, LdapConn, Scope, SearchEntry, SearchOptions};

mod bw_admin;
mod config;

fn main() {
    let config = config::Config::from_file();
    let mut client = bw_admin::Client::new(
        config.get_bitwarden_url().clone(),
        config.get_bitwarden_admin_token().clone(),
    );

    /*
     * let auth_response = client.auth();
     * println!("Auth Response: {:?}", auth_response);
     */

    match do_search(&config) {
        Ok(_) => (),
        Err(e) => println!("{}", e),
    }

    if let Err(e) = invite_from_ldap(&config, &mut client) {
        println!("{}", e);
    }

    /*
     * if let Err(e) = start_sync_loop(&config, %mut client) {
     *     println!("{}", e);
     * }
     */
}

/// Creates an LDAP connection, authenticating if necessary
fn ldap_client(ldap_url: String, bind_dn: String, bind_pw: String) -> Result<LdapConn, Box<Error>> {
    let ldap = LdapConn::new(ldap_url.as_str())?;
    match ldap.simple_bind(bind_dn.as_str(), bind_pw.as_str()) {
        _ => {}
    };

    Ok(ldap)
}

/// Retrieves search results from ldap
fn search_entries(config: &config::Config) -> Result<Vec<SearchEntry>, Box<Error>> {
    let ldap = ldap_client(
        config.get_ldap_url(),
        config.get_ldap_bind_dn(),
        config.get_ldap_bind_password(),
    );

    let mail_field = config.get_ldap_mail_field();
    let fields = vec!["uid", "givenname", "sn", "cn", mail_field.as_str()];

    // TODO: Something something error handling
    let (results, _res) = ldap?
        .with_search_options(SearchOptions::new().deref(DerefAliases::Always))
        .search(
            &config.get_ldap_search_base_dn().as_str(),
            Scope::Subtree,
            &config.get_ldap_search_filter().as_str(),
            fields,
        )?
        .success()?;

    // Build list of entries
    let mut entries = Vec::new();
    for result in results {
        entries.push(SearchEntry::construct(result));
    }

    Ok(entries)
}

/// Perform a simple search and list users
fn do_search(config: &config::Config) -> Result<(), Box<Error>> {
    let mail_field = config.get_ldap_mail_field();
    let entries = search_entries(config)?;
    for user in entries {
        println!("{:?}", user);
        if let Some(user_email) = user.attrs[mail_field.as_str()].first() {
            println!("{}", user_email);
        }
    }

    Ok(())
}

fn invite_from_ldap(
    config: &config::Config,
    client: &mut bw_admin::Client,
) -> Result<(), Box<Error>> {
    let mail_field = config.get_ldap_mail_field();
    for ldap_user in search_entries(config)? {
        if let Some(user_email) = ldap_user.attrs[mail_field.as_str()].first() {
            println!("Try to invite user: {}", user_email);
            let response = client.invite(user_email);
            println!("Invite response: {:?}", response);
        }
    }

    Ok(())
}

/*
 * fn start_sync_loop(config: &config::Config) -> Result<(), Box<Error>> {
 *     let interval = Duration::from_secs(config.get_ldap_sync_interval_seconds());
 *     loop {
 *         invite_from_ldap(config)?;
 *         sleep(interval);
 *     }
 * }
 */
