# ❌ OCI-003: sin HEALTHCHECK
# ❌ OCI-004: corre como root (no hay USER nonroot)
# ❌ OCI-005: sin labels OCI obligatorios

FROM rust:slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/rust-svc-bad /usr/local/bin/rust-svc-bad

EXPOSE 3001

# ❌ No hay USER nonroot
# ❌ No hay HEALTHCHECK
# ❌ No hay LABEL con metadatos OCI

ENTRYPOINT ["/usr/local/bin/rust-svc-bad"]
