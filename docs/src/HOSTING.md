## Proxy

## Requirements
- Set the  `SERVER_URL` environment variable to the url of the proxy.
- Turn on websocket support in your proxy

You won't be able to use your service via the plain local url as the websocket connection will fail.

If the SERVER_URL starts with
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