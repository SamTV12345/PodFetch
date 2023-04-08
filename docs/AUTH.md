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

| Variable          | Description                           | example                                                         |
|-------------------|---------------------------------------|-----------------------------------------------------------------|
| OIDC_AUTH         | Flag if OIDC should be enabled        | true                                                            |
| OIDC_AUTHORITY    | The url of the OIDC authority.        | Keycloak Master <keycloak-url/realms/master                     |
| OIDC_CLIENT_ID    | The client id of the OIDC client.     | podfetch                                                        |
| OIDC_REDIRECT_URI | The client secret of the OIDC client. | <your-server-url>/ui/login                                      |
| OIDC_SCOPE        | The scope of the oidc token           | This has a default value of "openid profile email"              |
| OIDC_JWKS         | The JWKS token uri                    | For Keycloak it is /realms/master/protocol/openid-connect/certs |       