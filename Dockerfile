# Stage 1: Frontend build
FROM node:20 AS frontend

WORKDIR /app

COPY package*.json ./
COPY vite.config.ts ./
COPY tsconfig.json ./
COPY ./src ./src
COPY ./public ./public

RUN npm install
RUN npm run build:web

# Stage 2: Backend build
FROM rust:1.78 AS backend

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src-tauri/ src-tauri/

RUN cargo build --release --manifest-path src-tauri/Cargo.toml

# Stage 3: Final container
FROM debian:bullseye-slim

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=backend /app/target/release/web_server /usr/local/bin/web_server
COPY --from=frontend /app/dist /dist

WORKDIR /dist

EXPOSE 3000

CMD ["/usr/local/bin/web_server"]
