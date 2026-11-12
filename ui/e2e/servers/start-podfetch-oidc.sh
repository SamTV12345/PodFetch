#!/usr/bin/env bash
# Second podfetch instance for oidc.spec.ts: OIDC login against the strict
# mock provider (mock-oidc.mjs on :9997), redirect URI on the login page as
# the docs used to recommend (#2315). Endpoints are not configured here -
# they have to come from the provider's discovery document.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"

export E2E_RUN_DIR="$ROOT/.e2e-run-oidc"
export E2E_PORT=8001
export E2E_SERVER_URL="http://127.0.0.1:8001"

export OIDC_AUTH=true
export OIDC_AUTHORITY="http://127.0.0.1:9997"
export OIDC_CLIENT_ID="podfetch"
export OIDC_REDIRECT_URI="http://127.0.0.1:8001/ui/login"
export OIDC_SCOPE="openid profile email"
export OIDC_JWKS="http://127.0.0.1:9997/.well-known/jwks.json"
unset OIDC_AUTHORIZATION_ENDPOINT OIDC_TOKEN_ENDPOINT

exec "$(dirname "$0")/start-podfetch.sh"
