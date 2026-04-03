## Proxy

PodFetch automatically derives its public URL from the incoming request headers. There is no need to configure a server URL manually.

## Requirements
- Ensure your reverse proxy forwards the following headers to PodFetch:
  - `X-Forwarded-Host` — the public hostname (e.g. `podfetch.example.com`)
  - `X-Forwarded-Proto` — the protocol (`https` or `http`)
  - `X-Forwarded-Prefix` — the sub-path, if PodFetch is hosted under a sub-directory (e.g. `/podfetch`)
- Turn on websocket support in your proxy

If you host PodFetch under a sub-path without a reverse proxy that sets `X-Forwarded-Prefix`, you can set the `SUB_DIRECTORY` environment variable instead (e.g. `SUB_DIRECTORY=/podfetch`).

The websocket protocol is determined from the forwarded protocol:
- https => Secured Websocket (wss)
- http => Unsecured Websocket (ws)

# Telegram

PodFetch can also send messages via Telegram if a new episode was downloaded.

To enable it you need to set the following variables:

| Variable             | Description                                                    | example                          |
|----------------------|----------------------------------------------------------------|----------------------------------|
| TELEGRAM_BOT_TOKEN   | The Bot token that you can acquire from Botfather with /newbot | asdj23:hsifuhi234klerlf...sadasd |
| TELEGRAM_BOT_CHAT_ID | The chat id of the chat where the bot should send the messages | 123456789                        |
| TELEGRAM_API_ENABLED | If the telegram api should be enabled.                         | true                             |

You can acquire the Telegram Bot chat id with the following steps:
1. Write a message to the bot
2. Open the following url in your browser: [Telegram API page](https://api.telegram.org/bot<TELEGRAM_BOT_TOKEN>/getUpdates)
3. Search for the chat id in the response


# Proxying requests to the podcast servers

In some cases you also need to proxy the traffic from the PodFetch server via a proxy. For that exists the `PODFETCH_PROXY` variable. You set it to the address of your proxy.