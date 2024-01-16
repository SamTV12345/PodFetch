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

### Keycloak

| Variable          | Description                                                   | Example                                                      |
|-------------------|---------------------------------------------------------------|--------------------------------------------------------------|
| OIDC_AUTH         | Flag if OIDC should be enabled                                | `true`                                                       |
| OIDC_AUTHORITY    | The url of the OIDC authority.                                | `<keycloak-url>/realms/master`                               |
| OIDC_CLIENT_ID    | The client id of the OIDC client.                             | `podfetch`                                                   |
| OIDC_REDIRECT_URI | The URI the OIDC authority redirects to after authentication. | `<your-server-url>/ui/login`                                 |
| OIDC_SCOPE        | The scope of the oidc token                                   | `openid profile email`                                       |
| OIDC_JWKS         | The JWKS token uri                                            | `<keycloak-url>/realms/master/protocol/openid-connect/certs` |

Note: For OIDC authorities that allow for selecting between `Confidential`/`Private` and `Public` for the Client Type (for example Authentik), use `Public`, as PodFetch does not need a client secret.

### Authelia

This assumes you already have OIDC set up in Authelia and your Authelia instance is being served on a subdomain `https://auth.DOMAIN.COM` with podfetch being served on it's own subdomain at `https://podfetch.DOMAIN.COM`

**Podfetch Configuration**

| Variable          | Description                                                   | Example                                                      |
|-------------------|---------------------------------------------------------------|--------------------------------------------------------------|
| OIDC_AUTH         | Flag if OIDC should be enabled                                | `true`                                                       |
| OIDC_AUTHORITY    | The url of the OIDC authority.                                | `https://auth.DOMAIN.COM`                                    |
| OIDC_CLIENT_ID    | The client id of the OIDC client.                             | `podfetch`                                                   |
| OIDC_REDIRECT_URI | The URI the OIDC authority redirects to after authentication. | `https://podfetch.DOMAIN.COM/ui/login`                       |
| OIDC_SCOPE        | The scope of the oidc token                                   | `openid profile email`                                       |
| OIDC_JWKS         | The JWKS token uri                                            | `https://auth.DOMAIN.COM/jwks.json` |

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
          - https://podfetch.DOMAIN.COM/ui/login
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