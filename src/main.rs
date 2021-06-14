extern crate anyhow;
extern crate ldap3;

use std::collections::HashSet;
use std::thread::sleep;
use std::time::Duration;

use anyhow::Context as _;
use anyhow::Error as AnyError;
use anyhow::Result;
use ldap3::{DerefAliases, LdapConn, LdapConnSettings, Scope, SearchEntry, SearchOptions};

mod config;
mod vw_admin;

fn main() {
    let config = config::Config::from_file();
    let mut client = vw_admin::Client::new(
        config.get_vaultwarden_url(),
        config.get_vaultwarden_admin_token(),
        config.get_vaultwarden_root_cert_file(),
    );

    invite_users(&config, &mut client, config.get_ldap_sync_loop())
}

/// Invites new users to Bitwarden from LDAP
fn invite_users(config: &config::Config, client: &mut vw_admin::Client, start_loop: bool) {
    if start_loop {
        start_sync_loop(config, client).expect("Failed to start invite sync loop");
    } else {
        invite_from_ldap(config, client).expect("Failed to invite users");
    }
}

/// Creates set of email addresses for users that already exist in Bitwarden
fn get_existing_users(client: &mut vw_admin::Client) -> Result<HashSet<String>, AnyError> {
    let all_users = client
        .users()
        .context("Could not get list of existing users from server")?;
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
) -> Result<LdapConn, AnyError> {
    let settings = LdapConnSettings::new()
        .set_starttls(starttls)
        .set_no_tls_verify(no_tls_verify);
    let ldap = LdapConn::with_settings(settings, ldap_url.as_str())
        .context("Failed to connect to LDAP server")?;
    ldap.simple_bind(bind_dn.as_str(), bind_pw.as_str())
        .context("Could not bind to LDAP server")?;

    Ok(ldap)
}

/// Retrieves search results from ldap
fn search_entries(config: &config::Config) -> Result<Vec<SearchEntry>, AnyError> {
    let ldap = ldap_client(
        config.get_ldap_url(),
        config.get_ldap_bind_dn(),
        config.get_ldap_bind_password(),
        config.get_ldap_no_tls_verify(),
        config.get_ldap_starttls(),
    )
    .context("LDAP client initialization failed")?;

    let mail_field = config.get_ldap_mail_field();
    let fields = vec!["uid", "givenname", "sn", "cn", mail_field.as_str()];

    // TODO: Something something error handling
    let (results, _res) = ldap
        .with_search_options(SearchOptions::new().deref(DerefAliases::Always))
        .search(
            &config.get_ldap_search_base_dn().as_str(),
            Scope::Subtree,
            &config.get_ldap_search_filter().as_str(),
            fields,
        )
        .context("LDAP search failure")?
        .success()
        .context("LDAP search usucessful")?;

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
) -> Result<(), AnyError> {
    let existing_users =
        get_existing_users(client).context("Failed to get existing users from server")?;
    let mail_field = config.get_ldap_mail_field();
    let mut num_users = 0;

    for ldap_user in search_entries(config)? {
        //
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
                client
                    .invite(user_email)
                    .context(format!("Failed to invite user {}", user_email))?;
                num_users += 1;
            }
        } else {
            match ldap_user.attrs.get("uid").and_then(|l| l.first()) {
                Some(user_uid) => println!(
                    "Warning: Email field, {:?}, not found on user {}",
                    mail_field, user_uid
                ),
                None => println!("Warning: Email field, {:?}, not found on user", mail_field),
            }
        }
    }

    // Maybe think about returning this value for some other use
    println!("Sent invites to {} user(s).", num_users);

    Ok(())
}

/// Begin sync loop to invite LDAP users to Bitwarden
fn start_sync_loop(config: &config::Config, client: &mut vw_admin::Client) -> Result<(), AnyError> {
    let interval = Duration::from_secs(config.get_ldap_sync_interval_seconds());
    let mut fail_count = 0;
    let fail_limit = 5;
    loop {
        if let Err(err) = invite_from_ldap(config, client) {
            println!(
                "Error inviting users from ldap. Count {}: {:?}",
                fail_count, err
            );
            fail_count += 1;
            if fail_count > fail_limit {
                return Err(err);
            }
        } else {
            fail_count = 0
        }

        sleep(interval);
    }
}
