## Basic Auth

Basic Auth is not required. If you use a reverse proxy like nginx you can use a better form that is also able to save passwords in your phone.
If you decide to use basic auth you need to set all three variables below. Otherwise, the container will crash with an error message as a safety measure.

| Variable   | Description                                                                 |
|------------|-----------------------------------------------------------------------------|
| BASIC_AUTH | Set to true if you want to use basic auth                                   |
| USERNAME   | Username for basic auth                                                     |
| PASSWORD   | Password for basic auth                                                     |

## OIDC

PodFetch also supports OIDC authentication. If you want to use it you need to set the following variables.  

If you enable it you need to disable BASIC_AUTH as it is not possible to use both at the same time.

Once you have created the user you intend to use as admin, you are then required to promote this user to admin via the command line.

Assuming your podfetch container is called `$PODFETCH` this can be done as follows, illustrating how the user `sam` is elevated to admin. (or uploader)

  -  Login with OIDC as user `sam`
  -  Run `docker exec -it $PODFETCH /app/podfetch users update`
  -  Enter the name `sam`
  -  Enter `role` 
  -  Enter `admin`
  -  Login as `sam` again and you should find `sam` is now an admin.

### Variables

| Variable                    | Description                                                                                                   | Example                                   |
|-----------------------------|---------------------------------------------------------------------------------------------------------------|-------------------------------------------|
| OIDC_AUTH                   | Flag if OIDC should be enabled                                                                                | `true`                                    |
| OIDC_AUTHORITY              | The issuer URL of your provider (the URL in front of `/.well-known/openid-configuration`).                     | `https://id.example.com`                  |
| OIDC_CLIENT_ID              | The client id of the OIDC client.                                                                             | `podfetch`                                |
| OIDC_REDIRECT_URI           | The URI the provider redirects to after authentication. Register exactly this URI at your provider.           | `<your-server-url>/ui/`                   |
| OIDC_SCOPE                  | The scope of the oidc token                                                                                   | `openid profile email`                    |
| OIDC_JWKS                   | The JWKS uri of your provider (`jwks_uri` in the discovery document)                                          | `https://id.example.com/.well-known/jwks.json` |
| OIDC_AUTHORIZATION_ENDPOINT | Optional. Authorization endpoint, if it can't be discovered.                                                  | `https://id.example.com/authorize`        |
| OIDC_TOKEN_ENDPOINT         | Optional. Token endpoint, if it can't be discovered.                                                          | `https://id.example.com/api/oidc/token`   |
| OIDC_REFRESH_INTERVAL       | Optional. How often (ms) the web UI refreshes the token. Default: 2 minutes.                                  | `120000`                                  |

**Endpoints.** On startup PodFetch reads `<OIDC_AUTHORITY>/.well-known/openid-configuration` and uses the
`authorization_endpoint` and `token_endpoint` from it. `OIDC_AUTHORIZATION_ENDPOINT` / `OIDC_TOKEN_ENDPOINT`
override the discovered values. Without a discovery document, PodFetch falls back to the previous behaviour:
`OIDC_AUTHORITY` is used as authorization endpoint and the token endpoint is its sibling `token`
(e.g. `.../protocol/openid-connect/auth` → `.../protocol/openid-connect/token`), so existing setups keep working.
The log shows which endpoints are used (`OIDC: discovered endpoints ...`).

**Redirect URI.** Both `<your-server-url>/ui/` and `<your-server-url>/ui/login` work. After the login you end up in the app.

**Client type.** PodFetch is a public client using the authorization code flow with PKCE (`S256`) and `state`;
it does not need a client secret. For providers that let you choose between `Confidential`/`Private` and
`Public` (for example Authentik or Pocket ID), use `Public`.

### Keycloak

| Variable          | Example                                                      |
|-------------------|--------------------------------------------------------------|
| OIDC_AUTH         | `true`                                                       |
| OIDC_AUTHORITY    | `<keycloak-url>/realms/master`                               |
| OIDC_CLIENT_ID    | `podfetch`                                                   |
| OIDC_REDIRECT_URI | `<your-server-url>/ui/`                                      |
| OIDC_SCOPE        | `openid profile email`                                       |
| OIDC_JWKS         | `<keycloak-url>/realms/master/protocol/openid-connect/certs` |

### Pocket ID

Create an OIDC client in Pocket ID with the callback URL `https://podfetch.DOMAIN.COM/ui/`, mark it as
public client and keep PKCE enabled.

| Variable          | Example                                             |
|-------------------|-----------------------------------------------------|
| OIDC_AUTH         | `true`                                              |
| OIDC_AUTHORITY    | `https://id.DOMAIN.COM`                             |
| OIDC_CLIENT_ID    | `<client id shown by Pocket ID>`                    |
| OIDC_REDIRECT_URI | `https://podfetch.DOMAIN.COM/ui/`                   |
| OIDC_SCOPE        | `openid profile email`                              |
| OIDC_JWKS         | `https://id.DOMAIN.COM/.well-known/jwks.json`       |

### Authelia

This assumes you already have OIDC set up in Authelia and your Authelia instance is being served on a subdomain `https://auth.DOMAIN.COM` with podfetch being served on it's own subdomain at `https://podfetch.DOMAIN.COM`

**Podfetch Configuration**

| Variable          | Example                              |
|-------------------|--------------------------------------|
| OIDC_AUTH         | `true`                               |
| OIDC_AUTHORITY    | `https://auth.DOMAIN.COM`            |
| OIDC_CLIENT_ID    | `podfetch`                           |
| OIDC_REDIRECT_URI | `https://podfetch.DOMAIN.COM/ui/`    |
| OIDC_SCOPE        | `openid profile email`               |
| OIDC_JWKS         | `https://auth.DOMAIN.COM/jwks.json`  |

**Authelia Configuration**

Configure the OIDC client in Authelia as below, you can change your `authorization_policy` and `consent_mode` according to your needs.

```yaml
      - id: podfetch
        description: Podfetch
        public: true
        authorization_policy: one_factor
        scopes:
          - openid
          - profile
          - email
        consent_mode: explicit
        redirect_uris:
          - https://podfetch.DOMAIN.COM/ui/
        userinfo_signing_algorithm: none
```


## Reverse Proxy

You can also use a reverse proxy like nginx to do the authentication. PodFetch supports this mode by setting the 
following variables:

| Variable                   | Description                                         | Example           |
|----------------------------|-----------------------------------------------------|-------------------|
| REVERSE_PROXY              | Flag if reverse proxy should be enabled             | `true`            |
| REVERSE_PROXY_HEADER       | The url of the reverse proxy.                       | `X-FORWARDED-FOR` |
| REVERSE_PROXY_AUTO_SIGN_UP | Flag if PodFetch should automatically sign up users | `true`            |

# User Creation

You can create an admin, user, or uploader either through [CLI](docs/CLI.md) or via invites.

To generate an invite, log into Podfetch → Top Right Icon → User Administration → Invites