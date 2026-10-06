# ThreatFlux Rust Dockerfile
# Multi-stage build for single-crate or workspace-based applications.
#
# Follows the ThreatFlux/rust-cicd-template Dockerfile: a Debian 13 (trixie)
# Rust builder and a distroless Debian 13 runtime. The runtime has no shell,
# no package manager and no coreutils; unifi-cli needs none of them (TLS is
# rustls, so it only needs glibc, libgcc and the CA bundle distroless ships).
#
# Base images are pinned by digest for reproducibility (Scorecard Pinned-Dependencies).
# Refresh with: docker buildx imagetools inspect <image> | awk '/^Digest:/{print $2}'

# rust:1.99.0-trixie (Debian 13), multi-arch index digest
FROM rust:1.99.0-trixie@sha256:15ad267e7a4cb2dce5905c90c76765adb6714945c5ea6d7c82673897a5e4067b AS rust-base

ARG VERSION=0.0.0
ARG BUILD_DATE=unknown
ARG VCS_REF=unknown
ARG BINARY_NAME=unifi-cli
ARG BINARY_PACKAGE=
ARG CLI_NAME=unifi-cli
ARG SBOM_MANIFEST_PATH=Cargo.toml
ARG OCI_IMAGE_TITLE="ThreatFlux UniFi SDK"
ARG OCI_IMAGE_DESCRIPTION="UniFi SDK CLI for UDM Pro and UniFi OS device automation"
ARG OCI_IMAGE_VENDOR=ThreatFlux
ARG OCI_IMAGE_SOURCE=https://github.com/ThreatFlux/threatflux-unifi-sdk

# tini is installed here so the runtime stage can copy it out: distroless ships
# no init, and PID 1 must reap zombies and forward signals. Exact Debian package
# revisions are not pinned; they follow the repositories of the pinned base.
# hadolint ignore=DL3008
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    pkg-config \
    libssl-dev \
    tini \
    && rm -rf /var/lib/apt/lists/*

FROM rust-base AS builder

RUN useradd -m -u 1000 builder
USER builder
WORKDIR /build

ENV CARGO_HOME=/home/builder/.cargo
ENV PATH="/home/builder/.cargo/bin:${PATH}"

COPY --chown=builder:builder . .

RUN rustc --version --verbose && cargo --version && \
    if [ -n "${BINARY_PACKAGE}" ]; then \
      cargo build --locked --release -p "${BINARY_PACKAGE}" --bin "${BINARY_NAME}" --all-features; \
    else \
      cargo build --locked --release --bin "${BINARY_NAME}" --all-features || cargo build --locked --release --all-features; \
    fi

# cargo-cyclonedx writes the SBOM beside the manifest it was handed, which in a
# workspace is not necessarily /build. The find normalizes the output location.
RUN cargo install cargo-cyclonedx --locked --version 0.5.8 && \
    cargo cyclonedx \
      --manifest-path "${SBOM_MANIFEST_PATH}" \
      --all-features \
      --format json \
      --spec-version 1.5 \
      --override-filename "${BINARY_NAME}-sbom" && \
    find /build -name "${BINARY_NAME}-sbom.json" -exec cp {} /build/sbom.cdx.json \; -quit && \
    test -s /build/sbom.cdx.json

# Stage the runtime filesystem under a fixed layout. The binary is staged as
# `app` at a fixed path because exec-form ENTRYPOINT and HEALTHCHECK do not
# expand build ARGs; a symlink keeps the friendly CLI name available. The
# writable working directory is created here because the distroless runtime
# has no shell to mkdir with; ownership is applied by COPY --chown.
RUN mkdir -p /home/builder/out/bin /home/builder/out/doc /home/builder/runtime-skel/data && \
    cp "target/release/${BINARY_NAME}" /home/builder/out/bin/app && \
    if [ "${CLI_NAME}" != "app" ]; then \
      ln -s app "/home/builder/out/bin/${CLI_NAME}"; \
    fi && \
    cp /build/sbom.cdx.json /home/builder/out/doc/sbom.cdx.json

# gcr.io/distroless/cc-debian13:nonroot (Debian 13), multi-arch index digest
FROM gcr.io/distroless/cc-debian13:nonroot@sha256:e792ab3d241a468a4fd7519ddbbebe66b49b5f365771716ea688ad40b6c6f1c2 AS runtime

ARG VERSION=0.0.0
ARG BUILD_DATE=unknown
ARG VCS_REF=unknown
ARG OCI_IMAGE_TITLE="ThreatFlux UniFi SDK"
ARG OCI_IMAGE_DESCRIPTION="UniFi SDK CLI for UDM Pro and UniFi OS device automation"
ARG OCI_IMAGE_VENDOR=ThreatFlux
ARG OCI_IMAGE_SOURCE=https://github.com/ThreatFlux/threatflux-unifi-sdk
ARG OCI_IMAGE_LICENSES=MIT
ARG OCI_IMAGE_DOCUMENTATION=https://docs.rs/threatflux-unifi-sdk

LABEL org.opencontainers.image.title="${OCI_IMAGE_TITLE}" \
      org.opencontainers.image.description="${OCI_IMAGE_DESCRIPTION}" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.created="${BUILD_DATE}" \
      org.opencontainers.image.revision="${VCS_REF}" \
      org.opencontainers.image.vendor="${OCI_IMAGE_VENDOR}" \
      org.opencontainers.image.source="${OCI_IMAGE_SOURCE}" \
      org.opencontainers.image.licenses="${OCI_IMAGE_LICENSES}" \
      org.opencontainers.image.documentation="${OCI_IMAGE_DOCUMENTATION}"

COPY --from=builder /usr/bin/tini /usr/bin/tini

# The binary and SBOM stay root-owned (0755/0644), so the runtime user can run
# and read them but not modify them; only the working directory is writable.
COPY --from=builder --chown=0:0 /home/builder/out/bin/ /usr/local/bin/
COPY --from=builder --chown=0:0 /home/builder/out/doc/ /usr/share/doc/app/
COPY --from=builder --chown=65532:65532 /home/builder/runtime-skel/data /data

# distroless "nonroot" user and group.
USER 65532:65532
WORKDIR /data

# Exec form (there is no shell in distroless); a nonzero exit means unhealthy.
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/app", "--version"]

ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/app"]
CMD []
