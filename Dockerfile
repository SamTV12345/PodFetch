FROM node:alpine as ui-builder

WORKDIR /app
COPY ./ui/ ./
RUN  npm install && npm run build


FROM rust:alpine3.17 as dependency-cache
USER root

RUN apk add pkgconfig openssl-dev libc-dev
WORKDIR /app/src

ADD Cargo.lock .
ADD Cargo.toml .
ADD dummy.rs ./src/main.rs
RUN RUSTFLAGS='-C target-feature=-crt-static' cargo build --release


FROM rust:alpine3.17 as builder

USER root

WORKDIR /app/src

RUN apk add pkgconfig openssl-dev libc-dev


COPY --from=dependency-cache /usr/local/cargo /usr/local/cargo
COPY --from=dependency-cache /app/src/target/ /app/src/target/
COPY --from=ui-builder /app/dist /app/src/static
RUN rm -rf /app/src/target/release/deps/podfetch*
RUN rm -rf /app/src/target/release/podfetch*

ADD Cargo.lock .
ADD Cargo.toml .
ADD static ./static
ADD migrations ./migrations
ADD db ./db
ADD src ./src
RUN RUSTFLAGS='-C target-feature=-crt-static' cargo build --release


FROM library/alpine:latest
WORKDIR /app/
RUN apk add libgcc tzdata
ENV TZ=Europe/Berlin

COPY --from=builder /app/src/target/release/podfetch /app/podfetch
COPY --from=builder /app/src/migrations /app/migrations
COPY --from=builder /app/src/db /app/db
COPY --from=ui-builder /app/dist /app/static
COPY ./docs/default.jpg /app/static/default.jpg

EXPOSE 8000
CMD ["./podfetch"]