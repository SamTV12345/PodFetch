# Pitfalls

## My server is not reachable from the internet

- Check your firewall
- Make sure you can ping the system

## My PodFetch server does not show any images

- Make sure your server is reachable and that your reverse proxy forwards the `X-Forwarded-Host`, `X-Forwarded-Proto`, and `X-Forwarded-Prefix` headers correctly

## I cannot login to the UI

- Make sure you have set up the `BASIC_AUTH` environment variable
- Make sure you have set up the `USERNAME` environment variable
- Make sure you have set up the `PASSWORD` environment variable
- Otherwise, reset your password via the CLI

## I can't stream any podcasts with authentication enabled

- Make sure your user has an api key
- Otherwise, generate one via the UI in the profile tab.