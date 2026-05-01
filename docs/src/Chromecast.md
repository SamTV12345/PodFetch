# Chromecast

PodFetch can play podcast episodes on Chromecast (Google Cast) devices on
the local network. Two deployment modes are supported:

1. **Direct** — the PodFetch server runs on the same LAN as the
   Chromecast and speaks the CAST protocol itself.
2. **Agent relay** — the PodFetch server is hosted somewhere remote
   (a public instance, a VPS, a friend's server) and a small `podfetch
   --agent` process running on the user's home LAN forwards Chromecast
   control commands to local devices.

The relay mode lets you use a hosted PodFetch UI without exposing your
home network or running PodFetch in two places.

## Permission model

Each Chromecast device has one of two `kind` values:

- **`chromecast_personal`** — visible only to the user that owns it.
  Devices reported by an agent default to this kind, owned by the user
  whose API key the agent authenticated with.
- **`chromecast_shared`** — household device, visible to every user on
  the instance. An admin promotes a device from personal to shared.

When you cast an episode, you can pick from your personal devices plus
every shared device. Active sessions are owned by the user who started
them — only that user can pause/stop/seek; a different user starting a
session on the same device implicitly stops the previous one.

## Running an agent

A user on the remote instance must already have an API key. (Run
`podfetch users` to manage them — see [CLI usage](./CLI.md).)

On a machine on the home LAN that can see the Chromecasts:

```bash
podfetch --agent \
  --remote https://podfetch.example.com \
  --api-key YOUR_USER_API_KEY \
  --agent-id home-lan
```

Flags:

| Flag           | Required | Default          | Description                                           |
|----------------|----------|------------------|-------------------------------------------------------|
| `--remote`     | yes      | —                | Base URL of the PodFetch instance to connect to.      |
| `--api-key`    | yes      | —                | Existing user API key on the remote instance.         |
| `--agent-id`   | no       | random UUID      | Stable id; pass the same one across restarts so the server keeps a single agent identity. |
| `--proxy-port` | no       | `8011`           | Reserved for the local episode-byte proxy (planned).  |

The agent:

- browses the local network for `_googlecast._tcp.local.` services
  (Chromecast mDNS records),
- connects to `wss?://<remote>/agent/ws` with a `Bearer` token,
- pushes the discovered device list to the server, and
- forwards `Play` requests to the local Chromecast(s).

It reconnects with exponential backoff (1s → 2s → 4s … capped at 60s)
if the websocket drops.

## What works today

- Discovery: Chromecasts on the agent's LAN appear in the UI as
  personal devices owned by the agent's user.
- Play: starting playback on a Chromecast works end-to-end. The Default
  Media Receiver app is launched and the episode URL is loaded.
- Pause / Resume / Stop / Seek / SetVolume: forwarded to the receiver
  via the per-session worker that holds the CAST connection.
- Live status streamback: the agent polls the receiver every ~1.5s and
  pushes Status updates over the agent websocket. The UI's progress bar
  reflects what the Chromecast actually plays, including external
  pauses or end-of-stream.
- Watchtime sync: every status update from the Chromecast is also
  persisted via the existing watchtime store, so the listened position
  during a cast session is visible everywhere PodFetch shows progress
  (episode resume, last-watched list, gPodder sync).
- Permission resolution: per-user vs household visibility is enforced
  on every API call.

## Known limitations

- **Codec transcoding**: PodFetch does not transcode for Chromecast.
  Common podcast codecs (MP3, AAC) work; uncommon ones may fail at the
  receiver.
- **Public-instance episode reachability**: when running in agent
  relay mode, the Chromecast still fetches the audio URL itself. If
  the public PodFetch URL is not reachable from the home LAN, casting
  will fail. A local agent proxy that re-serves bytes is on the roadmap
  but not yet implemented (the `--proxy-port` flag is reserved for it).

## Troubleshooting

**Agent connects but no devices appear.**
The mDNS browser only sees devices on the same broadcast domain.
Check the agent and Chromecast are on the same VLAN, that mDNS isn't
filtered by the router, and that no firewall is blocking UDP port 5353
on the agent host.

**Playback fails with "device unreachable" or a TLS error.**
Chromecasts present a self-signed certificate; PodFetch already
disables hostname verification for that case. The remaining failure
mode is the audio URL itself — make sure the URL the server sends is
reachable from the Chromecast.

**Agent disconnects every few minutes.**
Likely an HTTP proxy or load balancer in front of the remote PodFetch
that idles long-lived websockets. Configure a longer idle timeout, or
put PodFetch directly behind a TCP-level proxy.
