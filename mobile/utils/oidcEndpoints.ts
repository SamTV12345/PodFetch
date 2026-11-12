import type { components } from '@/schema';

type OidcConfig = components['schemas']['OidcConfig'];

// Newer servers send the endpoints (from OIDC_AUTHORIZATION_ENDPOINT /
// OIDC_TOKEN_ENDPOINT or the provider's discovery document). Older servers
// don't; fall back to the Keycloak paths this app always used.
export const oidcAuthorizationEndpoint = (config: OidcConfig) =>
    config.authorizationEndpoint || `${config.authority}/protocol/openid-connect/auth`;

export const oidcTokenEndpoint = (config: OidcConfig) =>
    config.tokenEndpoint || `${config.authority}/protocol/openid-connect/token`;
