// Minimal, deliberately strict OIDC provider for the playwright suite
// (modelled on Pocket ID / Ory, see issue #2315). No dependencies.
//
//  - discovery document at /.well-known/openid-configuration
//  - asymmetric endpoints: /authorize and /api/oidc/token (a client that
//    guesses `<authorization endpoint>/../token` ends up at /token -> 404)
//  - /authorize rejects requests without a sufficiently long `state`,
//    without PKCE S256 or with an unregistered redirect_uri
//  - /api/oidc/token checks code, redirect_uri, client_id and code_verifier
//    and returns an RS256 id_token whose key is published at /.well-known/jwks.json
//
// Test hooks: POST /__test/reject makes the next /authorize answer with
// `error=access_denied`; GET /__test/stats returns request counters;
// POST /__test/reset clears both.
import { createHash, generateKeyPairSync, randomBytes, sign } from 'node:crypto'
import { createServer } from 'node:http'

const PORT = 9997
const ISSUER = `http://127.0.0.1:${PORT}`
const CLIENT_ID = 'podfetch'
const REDIRECT_URIS = new Set(['http://127.0.0.1:8001/ui/login'])
const USERNAME = 'oidc-e2e-user'

const { privateKey, publicKey } = generateKeyPairSync('rsa', { modulusLength: 2048 })
const KID = 'e2e-key'
const jwk = { ...publicKey.export({ format: 'jwk' }), kid: KID, alg: 'RS256', use: 'sig' }

const codes = new Map()
const stats = { authorize: 0, token: 0, rejectedAuthorize: 0 }
let rejectNextAuthorize = false

const b64url = (input) => Buffer.from(input).toString('base64url')

const idToken = (nonce) => {
    const now = Math.floor(Date.now() / 1000)
    const header = b64url(JSON.stringify({ alg: 'RS256', typ: 'JWT', kid: KID }))
    const payload = b64url(JSON.stringify({
        iss: ISSUER,
        sub: 'e2e-subject',
        aud: CLIENT_ID,
        iat: now,
        exp: now + 3600,
        preferred_username: USERNAME,
        email: 'oidc-e2e@example.com',
        ...(nonce ? { nonce } : {}),
    }))
    const signature = sign('sha256', Buffer.from(`${header}.${payload}`), privateKey).toString('base64url')
    return `${header}.${payload}.${signature}`
}

const json = (res, status, body) => {
    res.writeHead(status, {
        'Content-Type': 'application/json',
        'Access-Control-Allow-Origin': '*',
        'Access-Control-Allow-Headers': 'Content-Type',
    })
    res.end(JSON.stringify(body))
}

const redirectWithError = (res, redirectUri, error, description, state) => {
    const target = new URL(redirectUri)
    target.searchParams.set('error', error)
    target.searchParams.set('error_description', description)
    if (state) target.searchParams.set('state', state)
    res.writeHead(302, { Location: target.toString() })
    res.end()
}

const readBody = (req) => new Promise((resolve) => {
    let data = ''
    req.on('data', (chunk) => { data += chunk })
    req.on('end', () => resolve(data))
})

const server = createServer(async (req, res) => {
    const url = new URL(req.url, ISSUER)

    if (req.method === 'OPTIONS') {
        return json(res, 204, {})
    }

    if (url.pathname === '/.well-known/openid-configuration') {
        return json(res, 200, {
            issuer: ISSUER,
            authorization_endpoint: `${ISSUER}/authorize`,
            token_endpoint: `${ISSUER}/api/oidc/token`,
            jwks_uri: `${ISSUER}/.well-known/jwks.json`,
            response_types_supported: ['code'],
            code_challenge_methods_supported: ['S256'],
        })
    }
    if (url.pathname === '/.well-known/jwks.json') {
        return json(res, 200, { keys: [jwk] })
    }

    if (url.pathname === '/authorize') {
        stats.authorize++
        const p = url.searchParams
        const redirectUri = p.get('redirect_uri') ?? ''
        if (p.get('client_id') !== CLIENT_ID || !REDIRECT_URIS.has(redirectUri)) {
            // Never redirect to an unregistered URI.
            stats.rejectedAuthorize++
            res.writeHead(400, { 'Content-Type': 'text/plain' })
            return res.end('invalid client_id or redirect_uri')
        }
        const state = p.get('state')
        if (!state || state.length < 8) {
            stats.rejectedAuthorize++
            return redirectWithError(res, redirectUri, 'invalid_state',
                'The state is missing or does not have enough characters and is therefore considered too weak.')
        }
        if (p.get('response_type') !== 'code' || p.get('code_challenge_method') !== 'S256' || !p.get('code_challenge')) {
            stats.rejectedAuthorize++
            return redirectWithError(res, redirectUri, 'invalid_request', 'PKCE with S256 is required.', state)
        }
        if (rejectNextAuthorize) {
            rejectNextAuthorize = false
            stats.rejectedAuthorize++
            return redirectWithError(res, redirectUri, 'access_denied', 'The user denied the request.', state)
        }
        const code = randomBytes(16).toString('hex')
        codes.set(code, {
            redirectUri,
            codeChallenge: p.get('code_challenge'),
            nonce: p.get('nonce'),
        })
        const target = new URL(redirectUri)
        target.searchParams.set('code', code)
        target.searchParams.set('state', state)
        target.searchParams.set('iss', ISSUER)
        res.writeHead(302, { Location: target.toString() })
        return res.end()
    }

    if (url.pathname === '/api/oidc/token' && req.method === 'POST') {
        stats.token++
        const body = new URLSearchParams(await readBody(req))
        if (body.get('client_id') !== CLIENT_ID) {
            return json(res, 401, { error: 'invalid_client' })
        }
        if (body.get('grant_type') === 'refresh_token') {
            return body.get('refresh_token')?.startsWith('rt-')
                ? json(res, 200, { id_token: idToken(), refresh_token: `rt-${randomBytes(8).toString('hex')}`, token_type: 'Bearer', expires_in: 3600 })
                : json(res, 400, { error: 'invalid_grant' })
        }
        const entry = codes.get(body.get('code') ?? '')
        if (body.get('grant_type') !== 'authorization_code' || !entry) {
            return json(res, 400, { error: 'invalid_grant' })
        }
        codes.delete(body.get('code'))
        if (body.get('redirect_uri') !== entry.redirectUri) {
            return json(res, 400, { error: 'invalid_grant', error_description: 'redirect_uri mismatch' })
        }
        const challenge = createHash('sha256').update(body.get('code_verifier') ?? '').digest('base64url')
        if (challenge !== entry.codeChallenge) {
            return json(res, 400, { error: 'invalid_grant', error_description: 'PKCE verification failed' })
        }
        return json(res, 200, {
            access_token: randomBytes(16).toString('hex'),
            id_token: idToken(entry.nonce),
            refresh_token: `rt-${randomBytes(8).toString('hex')}`,
            token_type: 'Bearer',
            expires_in: 3600,
        })
    }

    if (url.pathname === '/__test/reject' && req.method === 'POST') {
        rejectNextAuthorize = true
        return json(res, 200, {})
    }
    if (url.pathname === '/__test/reset' && req.method === 'POST') {
        rejectNextAuthorize = false
        stats.authorize = 0
        stats.token = 0
        stats.rejectedAuthorize = 0
        return json(res, 200, {})
    }
    if (url.pathname === '/__test/stats') {
        return json(res, 200, stats)
    }

    res.writeHead(404, { 'Content-Type': 'text/plain' })
    res.end('not found')
})

server.listen(PORT, '127.0.0.1', () => {
    console.log(`mock OIDC provider listening on ${ISSUER}`)
})
