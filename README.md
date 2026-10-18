# Podfetch

[![dependency status](https://deps.rs/repo/github/SamTV12345/PodFetch/status.svg)](https://deps.rs/repo/github/SamTV12345/PodFetch)
[![Build documentation](https://github.com/SamTV12345/PodFetch/actions/workflows/build-documentation.yml/badge.svg)](https://github.com/SamTV12345/PodFetch/actions/workflows/build-documentation.yml)
[![Build](https://github.com/SamTV12345/PodFetch/actions/workflows/pr-build.yml/badge.svg)](https://github.com/SamTV12345/PodFetch/actions/workflows/pr-build.yml)
<a href="https://repocloud.io/details/PodFetch/"><img src="https://dnk92k33or340.cloudfront.net/deploy-20px.png" alt="Deploy on RepoCloud" height="20"></a>
<a href="https://railway.com/deploy/podfetch-template"><img src="https://railway.com/button.svg" alt="Deploy on Railway" height="20"></a>

Podfetch is a self-hosted podcast manager.
It is a web app that lets you download podcasts and listen to them online.
It is written in Rust and uses React for the frontend.
It also contains a GPodder integration, so you can continue using your current podcast app.
It supports Podcasting 2.0 transcripts with full-text search across everything spoken in your episodes, and can generate missing transcripts via any OpenAI-compatible Whisper API.

Every time a new commit is pushed to the main branch, a new docker image is built and pushed to docker hub. So it is best to use something like [watchtower](https://github.com/containrrr/watchtower) to automatically update the docker image.


You can find the documentation with a UI preview [here](https://samtv12345.github.io/PodFetch/).


