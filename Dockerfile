# Multi-stage: build the WebAssembly game, then serve the static dist/ with nginx.
# This is the self-hosting path; GitHub Pages deploys the same dist/ without nginx.
#
#   docker build -t opensc .
#   docker run --rm -p 8080:80 opensc   # http://localhost:8080

FROM rust:1-bookworm AS build
RUN rustup target add wasm32-unknown-unknown \
    && apt-get update \
    && apt-get install -y --no-install-recommends nodejs \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY . .
RUN node scripts/build-web.mjs

FROM nginx:alpine
COPY --from=build /app/dist /usr/share/nginx/html
COPY web/nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
