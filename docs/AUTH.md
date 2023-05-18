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

| Variable          | Description                                                   | Example                                                      |
|-------------------|---------------------------------------------------------------|--------------------------------------------------------------|
| OIDC_AUTH         | Flag if OIDC should be enabled                                | `true`                                                       |
| OIDC_AUTHORITY    | The url of the OIDC authority.                                | `<keycloak-url>/realms/master`                               |
| OIDC_CLIENT_ID    | The client id of the OIDC client.                             | `podfetch`                                                   |
| OIDC_REDIRECT_URI | The URI the OIDC authority redirects to after authentication. | `<your-server-url>/ui/login`                                 |
| OIDC_SCOPE        | The scope of the oidc token                                   | `openid profile email`                                       |
| OIDC_JWKS         | The JWKS token uri                                            | `<keycloak-url>/realms/master/protocol/openid-connect/certs` |

Note: For OIDC authorities that allow for selecting between `Confidential`/`Private` and `Public` for the Client Type (for example Authentik), use `Public`, as PodFetch does not need a client secret.
