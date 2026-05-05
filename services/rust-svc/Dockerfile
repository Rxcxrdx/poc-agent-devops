# ── stage 1: build ──────────────────────────────────────────────────────────
FROM rust:slim AS builder

WORKDIR /app
COPY Cargo.toml ./
COPY src ./src

RUN cargo build --release

# ── stage 2: runtime ────────────────────────────────────────────────────────
FROM gcr.io/distroless/cc-debian12 AS runtime

# OCI labels inyectados en build time (requeridos por OCI-005)
ARG OCI_VERSION=0.1.0
ARG OCI_REVISION=unknown
ARG OCI_SOURCE=https://github.com/org/conformance-gate-poc
ARG CONFORMANCE_OWNER=platform-team
ARG CONFORMANCE_POLICY_VERSION=1.0.0

LABEL org.opencontainers.image.version="${OCI_VERSION}"
LABEL org.opencontainers.image.revision="${OCI_REVISION}"
LABEL org.opencontainers.image.source="${OCI_SOURCE}"
LABEL com.conformance.owner="${CONFORMANCE_OWNER}"
LABEL com.conformance.policy-version="${CONFORMANCE_POLICY_VERSION}"

COPY --from=builder /app/target/release/rust-svc /usr/local/bin/rust-svc

EXPOSE 3000

# OCI-004: nunca correr como root
USER nonroot

# OCI-003: HEALTHCHECK obligatorio
HEALTHCHECK --interval=15s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/rust-svc", "healthcheck"]

ENTRYPOINT ["/usr/local/bin/rust-svc"]
