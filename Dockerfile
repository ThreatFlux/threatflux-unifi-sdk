# ThreatFlux Rust Dockerfile
# Multi-stage build for single-crate or workspace-based applications.
#
# Builds a static PIE CLI with Alpine musl and Rustls, then ships it with
# static tini, current CA roots and a nonroot identity. No glibc, GCC shared
# runtime, zlib, shell or package manager is shipped. All features are retained.
#
# Base images are pinned by digest for reproducibility (Scorecard Pinned-Dependencies).
# Refresh with: docker buildx imagetools inspect <image> | awk '/^Digest:/{print $2}'

# Refresh trust data and Debian package provenance together.
FROM --platform=$BUILDPLATFORM debian:trixie-slim@sha256:62bd423aeee912639ca8ee681c2b762cb3e25137e0a92b20be455898a9f034ad AS trust-store
# hadolint ignore=DL3008
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
    dpkg --compare-versions "$(dpkg-query -W -f='${Version}' ca-certificates)" ge 20250419+deb13u1 && \
    dpkg-query -s ca-certificates > /ca-certificates.status && \
    rm -rf /var/lib/apt/lists/*

# Native musl builder; all CLI features use Rustls, with no OpenSSL requirement.
FROM rust:1.99.1-alpine3.24@sha256:557a94c82b485a14b569120e00867267108c734efe33f10cc31ca0cd5dd1eca9 AS rust-base

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

# Build tools remain in the builder. tini-static preserves init behavior.
RUN apk add --no-cache binutils build-base ca-certificates cmake linux-headers perl pkgconf tini-static

FROM rust-base AS builder

ARG CARGO_BUILD_JOBS=2

RUN adduser -D -u 1000 builder
USER builder
WORKDIR /build

ENV CARGO_HOME=/home/builder/.cargo
ENV PATH="/home/builder/.cargo/bin:${PATH}"

COPY --chown=builder:builder Cargo.toml Cargo.lock rust-toolchain.toml README.md LICENSE ./
COPY --chown=builder:builder src/ src/
COPY --chown=builder:builder examples/ examples/
COPY --chown=builder:builder tests/ tests/

# Use maintained Alpine musl, not Rust's older bundled libc archive. Keep
# only LLVM's unwinder from the bundled libraries; verify static PIE closure.
RUN rustc --version --verbose && cargo --version && \
    build_target="$(rustc -vV | sed -n 's/^host: //p')" && \
    mkdir -p /home/builder/native-unwind && \
    cp "$(rustc --print sysroot)/lib/rustlib/${build_target}/lib/self-contained/libunwind.a" /home/builder/native-unwind/ && \
    RUSTFLAGS="-C target-feature=+crt-static -C link-self-contained=no -C linker=cc -C relocation-model=pic -C link-arg=-static-pie -L native=/home/builder/native-unwind -C link-arg=-Wl,-Map=/home/builder/native-link.map" \
      cargo build --locked --release --target "${build_target}" \
        ${BINARY_PACKAGE:+-p "${BINARY_PACKAGE}"} --bin "${BINARY_NAME}" --all-features && \
    musl_archive="$(grep -oE '/[^[:space:](]+/libc\.a' /home/builder/native-link.map | sort -u)" && \
    test "$(readlink -f "${musl_archive}")" = /usr/lib/libc.a && \
    for executable in "target/${build_target}/release/${BINARY_NAME}" /sbin/tini-static; do \
      if readelf -l "${executable}" | grep -q INTERP || readelf -d "${executable}" | grep -q NEEDED; then \
        echo "Dynamic dependency in ${executable}" >&2; exit 1; \
      fi; \
    done && \
    cp "target/${build_target}/release/${BINARY_NAME}" "target/release-${BINARY_NAME}"

# cargo-cyclonedx writes the SBOM beside the manifest it was handed, which in a
# workspace is not necessarily /build. The find normalizes the output location.
RUN cargo install cargo-cyclonedx --locked --version 0.5.9 && \
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
    cp "target/release-${BINARY_NAME}" /home/builder/out/bin/app && \
    if [ "${CLI_NAME}" != "app" ]; then \
      ln -s app "/home/builder/out/bin/${CLI_NAME}"; \
    fi && \
    cp /build/sbom.cdx.json /home/builder/out/doc/sbom.cdx.json && \
    apk info --who-owns /usr/lib/libc.a /sbin/tini-static > /home/builder/out/doc/native-provenance.txt && \
    sha256sum /usr/lib/libc.a >> /home/builder/out/doc/native-provenance.txt

# gcr.io/distroless/static-debian13:nonroot, multi-arch index digest
FROM gcr.io/distroless/static-debian13:nonroot@sha256:516757106de35205efb70b91ba03f6f1298424a0678c603255cf2852a7000087 AS runtime

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

COPY --from=builder /sbin/tini-static /usr/bin/tini
COPY --from=trust-store /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=trust-store /ca-certificates.status /var/lib/dpkg/status.d/ca-certificates

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
