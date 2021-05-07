extern crate ldap3;

use std::collections::HashSet;
use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

use ldap3::{DerefAliases, LdapConn, LdapConnSettings, Scope, SearchEntry, SearchOptions};

mod config;
mod vw_admin;

fn main() {
    let config = config::Config::from_file();
    let mut client = vw_admin::Client::new(
        config.get_vaultwarden_url().clone(),
        config.get_vaultwarden_admin_token().clone(),
        config.get_vaultwarden_root_cert_file().clone(),
    );

    if let Err(e) = invite_users(&config, &mut client, config.get_ldap_sync_loop()) {
        panic!("{}", e);
    }
}

/// Invites new users to Bitwarden from LDAP
fn invite_users(
    config: &config::Config,
    client: &mut vw_admin::Client,
    start_loop: bool,
) -> Result<(), Box<dyn Error>> {
    if start_loop {
        start_sync_loop(config, client)?;
    } else {
        invite_from_ldap(config, client)?;
    }

    Ok(())
}

/// Creates set of email addresses for users that already exist in Bitwarden
fn get_existing_users(client: &mut vw_admin::Client) -> Result<HashSet<String>, Box<dyn Error>> {
    let all_users = client.users()?;
    let mut user_emails = HashSet::with_capacity(all_users.len());
    for user in all_users {
        user_emails.insert(user.get_email().to_lowercase());
        if user.is_disabled() {
            println!(
                "Existing disabled user found with email: {}",
                user.get_email()
            );
        } else {
            println!(
                "Existing user or invite found with email: {}",
                user.get_email()
            );
        }
    }

    Ok(user_emails)
}

/// Creates an LDAP connection, authenticating if necessary
fn ldap_client(
    ldap_url: String,
    bind_dn: String,
    bind_pw: String,
    no_tls_verify: bool,
    starttls: bool,
) -> Result<LdapConn, Box<dyn Error>> {
    let settings = LdapConnSettings::new()
        .set_starttls(starttls)
        .set_no_tls_verify(no_tls_verify);
    let ldap = LdapConn::with_settings(settings, ldap_url.as_str())?;
    match ldap.simple_bind(bind_dn.as_str(), bind_pw.as_str()) {
        _ => {}
    };

    Ok(ldap)
}

/// Retrieves search results from ldap
fn search_entries(config: &config::Config) -> Result<Vec<SearchEntry>, Box<dyn Error>> {
    let ldap = ldap_client(
        config.get_ldap_url(),
        config.get_ldap_bind_dn(),
        config.get_ldap_bind_password(),
        config.get_ldap_no_tls_verify(),
        config.get_ldap_starttls(),
    );

    if ldap.is_err() {
        println!("Error: Could not bind to ldap server");
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
    client: &mut vw_admin::Client,
) -> Result<(), Box<dyn Error>> {
    match get_existing_users(client) {
        Ok(existing_users) => {
            let mail_field = config.get_ldap_mail_field();
            let mut num_users = 0;
            for ldap_user in search_entries(config)? {
                // Safely get first email from list of emails in field
                if let Some(user_email) = ldap_user
                    .attrs
                    .get(mail_field.as_str())
                    .and_then(|l| (l.first()))
                {
                    if existing_users.contains(&user_email.to_lowercase()) {
                        println!("User with email already exists: {}", user_email);
                    } else {
                        println!("Try to invite user: {}", user_email);
                        // TODO: Validate response
                        let _response = client.invite(user_email);
                        num_users = num_users + 1;
                        // println!("Invite response: {:?}", response);
                    }
                } else {
                    println!("Warning: Email field, {:?}, not found on user", mail_field);
                }
            }

            // Maybe think about returning this value for some other use
            println!("Sent invites to {} user(s).", num_users);
        }
        Err(e) => {
            println!("Error: Failed to get existing users from Bitwarden");
            return Err(e);
        }
    }

    Ok(())
}

/// Begin sync loop to invite LDAP users to Bitwarden
fn start_sync_loop(
    config: &config::Config,
    client: &mut vw_admin::Client,
) -> Result<(), Box<dyn Error>> {
    let interval = Duration::from_secs(config.get_ldap_sync_interval_seconds());
    loop {
        invite_from_ldap(config, client)?;
        sleep(interval);
    }
}
