# sodmin Dockerfile.
#
# Build context: the cokret-dev umbrella directory (parent of sodmin).
# The Cargo workspace at sodmin/Cargo.toml has path-deps on sibling
# repos that must be present in the build sandbox:
#
#   - ../cokret-rust-sdk/crates/sdk      (cokret SDK)
#   - ../coauth/crates/admin-types        (coauth admin types)
#
# Standalone single-repo build (the sodmin docker.yml workflow) checks
# out coauth + cokret-rust-sdk into siblings of `./` and invokes:
#   docker build -f sodmin/Dockerfile ..
#
# The example-stack compose file does the equivalent via
#   build:
#     context: ../..
#     dockerfile: sodmin/Dockerfile

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

WORKDIR /workspace

# Copy the sibling path-deps FIRST so cargo fetch / dx build can resolve
# the workspace `path = "../<sibling>/..."` entries. Order matches the
# umbrella layout: cokret-dev/{cokret-rust-sdk,coauth,sodmin}/.
COPY cokret-rust-sdk/ /workspace/cokret-rust-sdk
COPY coauth/ /workspace/coauth
COPY sodmin/ /workspace/sodmin

WORKDIR /workspace/sodmin

# Pre-fetch deps so source-only changes don't redownload the world.
RUN --mount=type=cache,id=sodmin-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=sodmin-cargo-git,target=/usr/local/cargo/git \
    cargo fetch --locked

RUN --network=default \
    --mount=type=cache,id=sodmin-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=sodmin-cargo-git,target=/usr/local/cargo/git \
    --mount=type=cache,id=sodmin-target,target=/workspace/sodmin/target \
    for i in 1 2 3; do dx build --release --debug-symbols false && break || echo "Retry $i..." && sleep 10; done && \
    dist_dir="$(find /workspace/sodmin/target/dx -type d -path '*/release/web/public' | head -n 1)" && \
    test -n "$dist_dir" && \
    cp -r "$dist_dir" /workspace/dist

FROM nginx:alpine

COPY --from=builder /workspace/dist /usr/share/nginx/html
COPY --chmod=755 sodmin/docker-entrypoint.sh /docker-entrypoint.sh

# Runtime configuration. SOLAND_URL is the soland Principal Server (admin
# + reducer surface) the proxy should forward to; COAUTH_URL is the
# coauth admin service the proxy forwards to (with bearer); COAUTH_PUBLIC_URL
# is the browser-facing coauth origin used for OAuth2 redirects.
#
ENV SOLAND_URL=""
ENV COAUTH_URL=""
ENV COAUTH_PUBLIC_URL=""
ENV SODMIN_PORT="80"

EXPOSE 80
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD wget -q --spider "http://127.0.0.1:${SODMIN_PORT:-80}/healthz" || exit 1
CMD ["/docker-entrypoint.sh"]
