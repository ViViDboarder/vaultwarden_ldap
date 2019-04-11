extern crate ldap3;

use std::collections::HashSet;
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

    if let Err(e) = invite_users(&config, &mut client, config.get_ldap_sync_loop()) {
        panic!("{}", e);
    }
}

/// Invites new users to Bitwarden from LDAP
fn invite_users(
    config: &config::Config,
    client: &mut bw_admin::Client,
    start_loop: bool,
) -> Result<(), Box<Error>> {
    // TODO: Better error handling to differentiate failure to connect to Bitwarden vs LDAP

    if start_loop {
        start_sync_loop(config, client)?;
    } else {
        invite_from_ldap(config, client)?;
    }

    Ok(())
}

/// Creates set of email addresses for users that already exist in Bitwarden
fn get_existing_users(client: &mut bw_admin::Client) -> Result<HashSet<String>, Box<Error>> {
    let all_users = client.users_and_invites()?;
    let mut user_emails = HashSet::with_capacity(all_users.len());
    for user in all_users {
        user_emails.insert(user.get_email());
        if user.is_enabled() {
            println!("Existing user or invite found with email: {}", user.get_email());
        } else {
            println!("Existing disabled user found with email: {}", user.get_email());
        }
    }

    Ok(user_emails)
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

    if ldap.is_err() {
        println!("Error: Could not connect to ldap server");
    }

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

/// Invite all LDAP users to Bitwarden
fn invite_from_ldap(
    config: &config::Config,
    client: &mut bw_admin::Client,
) -> Result<(), Box<Error>> {
    let existing_users = get_existing_users(client)?;

    let mail_field = config.get_ldap_mail_field();
    let mut num_users = 0;
    for ldap_user in search_entries(config)? {
        if let Some(user_email) = ldap_user.attrs[mail_field.as_str()].first() {
            if existing_users.contains(user_email) {
                println!("User with email already exists: {}", user_email);
            } else {
                println!("Try to invite user: {}", user_email);
                let response = client.invite(user_email);
                num_users = num_users + 1;
                println!("Invite response: {:?}", response);
            }
        }
    }

    // Maybe think about returning this value for some other use
    println!("Sent invites to {} user(s).", num_users);

    Ok(())
}

/// Begin sync loop to invite LDAP users to Bitwarden
fn start_sync_loop(
    config: &config::Config,
    client: &mut bw_admin::Client,
) -> Result<(), Box<Error>> {
    let interval = Duration::from_secs(config.get_ldap_sync_interval_seconds());
    loop {
        invite_from_ldap(config, client)?;
        sleep(interval);
    }
}
