# bitwarden_rs_ldap
A simple LDAP connector for [bitwarden_rs](https://github.com/dani-garcia/bitwarden_rs)

After configuring, simply run `bitwarden_rs_ldap` and it will invite any users it finds in LDAP to your `bitwarden_rs` instance.

## Configuration

Configuration is read from a TOML file. The default location is `config.toml`, but this can be configured by setting the `CONFIG_PATH` env variable to whatever path you would like.

Configuration values are as follows:

|Name|Type|Optional|Description|
|----|----|--------|-----------|
|`bitwarden_url`|String||The root URL for accessing `bitwarden_rs`. Eg: `https://bw.example.com`|
|`bitwarden_admin_token`|String||The value passed as `ADMIN_TOKEN` to `bitwarden_rs`|
|`ldap_host`|String||The hostname or IP address for your ldap server|
|`ldap_scheme`|String|Optional|The that should be used to connect. `ldap` or `ldaps`. This is set by default based on SSL settings|
|`ldap_ssl`|Boolean|Optional|Indicates if SSL should be used. Defaults to `false`|
|`ldap_port`|Integer|Optional|Port used to connect to the LDAP server. This will default to 389 or 636, depending on your SSL settings|
|`ldap_bind_dn`|String||The dn for the bind user that will connect to LDAP. Eg. `cn=admin,dc=example,dc=org`|
|`ldap_bind_password`|String||The password for the provided bind user.|
|`ldap_search_base_dn`|String||Base dn that will be used when searching LDAP for users. Eg. `dc=example,dc=org`|
|`ldap_search_filter`|String||Filter used when searching LDAP for users. Eg. `(&(objectClass=*)(uid=*))`|
|`ldap_mail_field`|String|Optional|Field for each user record that contains the email address to use. Defaults to `mail`|
|`ldap_sync_interval_seconds`|Integer|Optional|Number of seconds to wait between each LDAP request. Defaults to `60`|
|`ldap_sync_loop`|Boolean|Optional|Indicates whether or not syncing should be polled in a loop or done once. Defaults to `true`|

## Future

* Any kind of proper logging
* Tests
