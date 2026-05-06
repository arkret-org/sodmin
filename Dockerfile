FROM --platform=$BUILDPLATFORM rust:bookworm AS builder

ENV CARGO_HTTP_TIMEOUT=600
ENV CARGO_HTTP_MULTIPLEXING=false
ENV CARGO_NET_RETRY=10
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

ENV RUSTUP_HTTP_TIMEOUT=600
# Install wasm target (with retries for flaky networks)
RUN --network=default \
    for i in 1 2 3 4 5; do rustup target add wasm32-unknown-unknown && break || sleep 10; done
# Pre-install binaryen
RUN apt-get update && apt-get install -y binaryen && rm -rf /var/lib/apt/lists/* || true

RUN --mount=type=cache,id=sodmin-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=sodmin-cargo-git,target=/usr/local/cargo/git \
    cargo install dioxus-cli@0.7.5 --locked

WORKDIR /app

COPY Cargo.toml Cargo.lock Dioxus.toml ./
RUN mkdir -p src && echo "fn main() {}" > src/main.rs
RUN --mount=type=cache,id=sodmin-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=sodmin-cargo-git,target=/usr/local/cargo/git \
    cargo fetch --locked

COPY . .
RUN --network=default \
    --mount=type=cache,id=sodmin-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=sodmin-cargo-git,target=/usr/local/cargo/git \
    --mount=type=cache,id=sodmin-target,target=/app/target \
    for i in 1 2 3; do dx build --release && break || echo "Retry $i..." && sleep 10; done && \
    dist_dir="$(find /app/target/dx -type d -path '*/release/web/public' | head -n 1)" && \
    test -n "$dist_dir" && \
    cp -r "$dist_dir" /app/dist

FROM nginx:alpine

COPY --from=builder /app/dist /usr/share/nginx/html
COPY --chmod=755 docker-entrypoint.sh /docker-entrypoint.sh

# Runtime configuration. SOLAND_URL is the soland Principal Server (admin
# + reducer surface) the proxy should forward to; COAUTH_URL is the
# coauth admin service the proxy forwards to (with bearer); COAUTH_PUBLIC_URL
# is the browser-facing coauth origin used for OAuth2 redirects.
#
# Legacy aliases (PALPO_URL / MATRIX_URL / PASION_URL / PASION_PUBLIC_URL /
# PADMIN_PORT) are still honored for one release cycle so existing
# deployments keep working.
ENV SOLAND_URL=""
ENV COAUTH_URL=""
ENV COAUTH_PUBLIC_URL=""
ENV SODMIN_PORT="80"

# Legacy aliases (kept for backward compatibility, will be removed in a
# future release). Prefer the SOLAND_URL / COAUTH_URL / COAUTH_PUBLIC_URL
# / SODMIN_PORT variables above.
ENV PALPO_URL=""
ENV MATRIX_URL=""
ENV PASION_URL=""
ENV PASION_PUBLIC_URL=""
ENV PADMIN_PORT=""

EXPOSE 80
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD wget -q --spider "http://127.0.0.1:${SODMIN_PORT:-${PADMIN_PORT:-80}}/healthz" || exit 1
CMD ["/docker-entrypoint.sh"]
