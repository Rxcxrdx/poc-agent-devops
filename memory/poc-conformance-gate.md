# PoC Conformance Gate — Hoja de Ruta Completa

## Documento de ejecución por fases para agente IA

> Versión final: 2026-05-04
> Stack: Rust (Axum) + SonarCloud + OpenCode SDK + Claude Sonnet 4.6 + GitHub Actions

---

## 🟢 ESTADO ACTUAL — sesión 2026-05-05

### Pipeline en producción (live en GitHub)

```
test-services  →  sonarcloud  →  conformance
   (auto)         (todo repo)    (auto, todos los servicios)
```

**3 jobs totales**, todos con auto-discovery de `services/*`. Agregar un servicio nuevo NO requiere editar `ci.yml`.

### Repos vivos

| Repo | URL | Estado |
|------|-----|--------|
| `Rxcxrdx/conformance-agent` | github.com/Rxcxrdx/conformance-agent | ✅ tag v2 (soporta `services-dir`) |
| `Rxcxrdx/poc-agent-devops` | github.com/Rxcxrdx/poc-agent-devops | ✅ CI corriendo |
| SonarCloud project | sonarcloud.io/project/overview?id=Rxcxrdx_poc-agent-devops | ✅ Org `devyzr`, key `Rxcxrdx_poc-agent-devops` |

### Qué analiza cada herramienta (resumen)

| Job | Herramienta | Detecta | NO detecta |
|-----|-------------|---------|------------|
| `test-services` | `cargo test` + `cargo clippy -D warnings` + `cargo-llvm-cov` | Errores compilación, lints estándar, genera lcov coverage | Reglas arquitecturales custom |
| `sonarcloud` | SonarCloud (Sonar way QG) | Bugs, vulnerabilities, security hotspots, code smells, cognitive complexity, duplicaciones, coverage % | BOX-001/002/003, OCI-003/004/005 |
| `conformance` | OpenCode SDK + Claude Sonnet 4.6 | BOX-001 (envelope ApiResponse), BOX-002 (.unwrap()/.expect()), BOX-003 (capas/arquitectura), OCI-003/004/005 (Dockerfile) | — |

### SonarCloud — qué métricas se publican

Configurado en `sonar-project.properties` (raíz del repo):
- **Project key**: `Rxcxrdx_poc-agent-devops`
- **Organization**: `devyzr`
- **Sources**: `services/` (todos los microservicios en un solo proyecto)
- **Coverage report**: `coverage/lcov.info` (generado por `cargo-llvm-cov` en el job de tests, descargado vía artifact)
- **Quality Gate**: `Sonar way` (default) — bloquea si: bugs nuevos, vulnerabilities nuevas, hotspots sin revisar, coverage nuevo < 80%, duplicación nueva > 3%

### Secrets configurados en GitHub

| Secret | Uso | Estado |
|--------|-----|--------|
| `SONAR_TOKEN` | Auth con sonarcloud.io | ✅ configurado |
| `ANTHROPIC_API_KEY` | Auth para OpenCode SDK + Claude | ⏳ pendiente del usuario |

### Próximos pasos pendientes

1. ⏳ Usuario añade `ANTHROPIC_API_KEY` como secret en GitHub
2. ⏳ Verificar pipeline completo: `test-services ✅ → sonarcloud ✅ → conformance` debe mostrar `rust-svc PASS / rust-svc-bad BLOCK`
3. ⏳ Configurar Branch Protection Rules en `main` para requerir los 3 jobs como required status checks
4. (Opcional) Crear servicio `services/nuevo-svc/` para demostrar auto-discovery sin tocar ci.yml

### Archivos clave del estado actual

| Archivo | Cambio reciente |
|---------|-----------------|
| `poc-agent-devops/.github/workflows/ci.yml` | Refactorizado a 3 jobs con auto-discovery + cobertura |
| `poc-agent-devops/sonar-project.properties` | Header documenta exactamente qué Sonar revisa y qué NO; añadido `coverageReportPaths` |
| `poc-agent-devops/services/rust-svc/src/error.rs` | `#[allow(dead_code)]` en variant `NotFound` para pasar clippy `-D warnings` |
| `conformance-agent/action.yml` | Nuevo input opcional `services-dir` (además de `service-path`) |
| `conformance-agent/src/index.ts` | Auto-descubre servicios cuando `INPUT_SERVICES_DIR` está set; decisión global `block` si cualquier servicio bloquea |

---

## ★ ESTADO ACTUAL — sesión 2026-05-04 (actualizado al cierre)

### Estructura de repos (DECISIÓN FINAL)

Se decidió usar **2 repos separados en GitHub** bajo el usuario `Rxcxrdx`:

```
GitHub:
├── Rxcxrdx/conformance-agent     ← la GitHub Action publicada (reutilizable por cualquier repo)
└── Rxcxrdx/poc-agent-devops      ← el POC que la consume
```

En disco local están en:

```
/Users/macbookpro/Desktop/AGENT FLOW/
├── conformance-agent/             ← será Rxcxrdx/conformance-agent en GitHub
└── poc-agent-devops/              ← será Rxcxrdx/poc-agent-devops en GitHub
```

### Por qué 2 repos y no uno

La GitHub Action (`conformance-agent`) es una herramienta reutilizable independiente.
El POC (`poc-agent-devops`) es el caso de uso que la consume.
Tenerlos separados permite publicar la Action con versión (`@v1`) y que cualquier otro repo
del equipo pueda usarla con `uses: Rxcxrdx/conformance-agent@v1` sin copiar código.

### Lo que está construido y listo ✅

```
conformance-agent/                         ← REPO 1 (será Rxcxrdx/conformance-agent)
├── action.yml                             define inputs (service-path) + outputs (decision, confidence)
├── package.json                           @opencode-ai/sdk (latest), @anthropic-ai/sdk
├── tsconfig.json
└── src/
    ├── types.ts                           Violation, ConformanceResult, Decision
    ├── dockerfile.ts                      checks OCI-003/004/005 sin LLM (determinista)
    ├── source.ts                          utilidad: lee .rs, elimina test blocks
    ├── agent.ts                           OpenCode SDK — agente explora repo con file tools
    ├── decision.ts                        pass | manual_review | block
    └── index.ts                           dual: CLI (--service) + GitHub Actions (INPUT_SERVICE_PATH)

poc-agent-devops/                          ← REPO 2 (será Rxcxrdx/poc-agent-devops)
├── CLAUDE.md                              reglas IDE (BOX-001..004, OCI, Dockerfile)
├── policy/conformity-policy.yaml          fuente de verdad de reglas
├── .github/workflows/ci.yml               pipeline 6 jobs — YA ACTUALIZADO a uses: Rxcxrdx/conformance-agent@v1
├── services/
│   ├── rust-svc/                          servicio CONFORME
│   │   ├── Cargo.toml                     axum 0.8, utoipa 5, reqwest 0.12, axum-test 20
│   │   ├── Dockerfile                     multi-stage + distroless + USER nonroot + HEALTHCHECK + labels OCI
│   │   ├── sonar-project.properties       ⚠️ SONAR_ORG pendiente de reemplazar con slug de Rxcxrdx
│   │   └── src/                           7/7 tests pasan ✅
│   └── rust-svc-bad/                      servicio NO CONFORME (violaciones intencionales)
│       ├── Cargo.toml
│       ├── Dockerfile                     ❌ sin HEALTHCHECK, sin USER nonroot, sin labels
│       ├── sonar-project.properties       ⚠️ SONAR_ORG pendiente de reemplazar
│       └── src/
│           ├── main.rs                    ❌ BOX-002 (.unwrap() en producción)
│           └── routes/news.rs             ❌ BOX-001 (sin envelope), BOX-003 (lógica inline)
└── memory/                                documentación de contexto (este archivo)
```

> ⚠️ NOTA: La carpeta `poc-agent-devops/conformance/` fue eliminada — era un prototipo
> incompleto anterior. La versión final y completa vive en `conformance-agent/`.

### Pipeline CI (6 jobs) — diseñado, pendiente primer push

```
test-rust-svc           cargo test + clippy   → ✅ pasa (7/7 tests)
test-rust-svc-bad       cargo test            → ✅ pasa (0 tests — demuestra limitación de cargo)
sonarcloud-rust-svc     SonarCloud            → ✅ pasa
sonarcloud-rust-svc-bad SonarCloud            → ✅ pasa (demuestra que Sonar NO detecta BOX-001/002/003)
conformance-rust-svc    uses: Rxcxrdx/conformance-agent@v1  → ✅ PASS esperado
conformance-rust-svc-bad uses: Rxcxrdx/conformance-agent@v1 → ❌ BLOCK esperado
```

### LLM elegido: Anthropic directa

- Se usa `ANTHROPIC_API_KEY` como GitHub Secret en `poc-agent-devops`
- El SDK de OpenCode llama a `anthropic/claude-sonnet-4-6` para las revisiones
- No se migró al SDK de GitHub Copilot — se mantiene `@opencode-ai/sdk`
- GitHub Copilot vía OpenCode requiere OAuth device flow (no compatible limpio con CI)

---

### ⚡ PENDIENTES — próxima sesión (en orden exacto)

**PASO 1 — Probar el agente localmente ANTES de hacer push** (recomendado)

```bash
cd "/Users/macbookpro/Desktop/AGENT FLOW/conformance-agent"
# ya tiene node_modules instalados
ANTHROPIC_API_KEY=sk-ant-... npx tsx src/index.ts \
  --service "../poc-agent-devops/services/rust-svc"
# Esperado: decision: pass, exit 0

ANTHROPIC_API_KEY=sk-ant-... npx tsx src/index.ts \
  --service "../poc-agent-devops/services/rust-svc-bad"
# Esperado: decision: block, exit 1
```

**PASO 2 — Crear repo `conformance-agent` en GitHub y hacer push con tag**

```bash
# 1. Ir a https://github.com/new → nombre: conformance-agent → Create (público)
cd "/Users/macbookpro/Desktop/AGENT FLOW/conformance-agent"
git init
git add .
git commit -m "feat: initial conformance-agent GitHub Action

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
git branch -M main
git remote add origin https://github.com/Rxcxrdx/conformance-agent.git
git push -u origin main
git tag v1
git push --tags
```

**PASO 3 — Configurar SonarCloud**

- Ir a https://sonarcloud.io → Login con GitHub → Import `poc-agent-devops`
- Obtener el **organization slug** (probablemente `rxcxrdx`)
- Editar estos 2 archivos reemplazando `SONAR_ORG_PLACEHOLDER`:
  - `poc-agent-devops/services/rust-svc/sonar-project.properties`
  - `poc-agent-devops/services/rust-svc-bad/sonar-project.properties`
- My Account → Security → Generate Token → copiar

**PASO 4 — Crear repo `poc-agent-devops` en GitHub con los 2 secrets**

```bash
# 1. Ir a https://github.com/new → nombre: poc-agent-devops → Create (público)
cd "/Users/macbookpro/Desktop/AGENT FLOW/poc-agent-devops"
git init
git add .
git commit -m "feat: initial POC conformance gate

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>"
git branch -M main
git remote add origin https://github.com/Rxcxrdx/poc-agent-devops.git
git push -u origin main
```

- Settings → Secrets and variables → Actions → New repository secret:
  - `ANTHROPIC_API_KEY` = tu key de console.anthropic.com
  - `SONAR_TOKEN` = token generado en paso 3

**PASO 5 — Verificar el pipeline** 🚀

- GitHub → Rxcxrdx/poc-agent-devops → Actions
- Resultado esperado:
  - 5 jobs verdes ✅
  - `conformance-rust-svc-bad` rojo ❌ (intencionalmente — demuestra el valor del agente)

---

## OBJETIVO

Construir un pipeline CI/CD que evalúe si un servicio Rust ("caja") cumple un estándar de conformidad antes de permitir el merge y deploy. El último paso del pipeline es un agente autónomo (OpenCode SDK) que decide pass / manual_review / block. Todo queda trazado en audit logs.

---

## ARQUITECTURA FINAL

```
[Dev IDE + Claude Code]
  └── construye → services/rust-svc/  (Rust/Axum — la "caja")
                        │
                      git push / PR
                        │
              [GitHub Actions CI]
                  │
                  ├── JOB 1: cargo test + clippy          (compilación + tests unitarios)
                  ├── JOB 2: cargo audit                   (SAST — CVEs supply chain, también pasa en rust-svc-bad)
                  ├── docker build + syft (SBOM) + cosign (firma, en JOB 3)
                  │
                  └── ★ CONFORMANCE GATE (último paso)
                          │
                          ├── Step A: conftest + Rego policies  (sin LLM)
                          │     └── block inmediato si hay critical
                          ├── Step B: OpenCode SDK agent        (Sonnet, checks cualitativos)
                          └── Step C: Claude Opus reflection    (solo zona ambigua)
                                  │
                          ┌───────┴────────┐
                          │  DECISIÓN JSON  │
                          └───────┬────────┘
                    pass ─────────┤
                    manual_review ┤──→ GitHub Environment (aprobación humana)
                    block ────────┘──→ pipeline falla, no hay deploy
```

---

## ESTRUCTURA DEL PROYECTO

```
conformance-gate-poc/
├── services/
│   ├── rust-svc/                    # caja CONFORME — escenarios 1, 2, 3, 5
│   │   ├── Cargo.toml
│   │   ├── Dockerfile
│   │   └── src/
│   │       ├── main.rs
│   │       ├── state.rs
│   │       ├── error.rs
│   │       ├── routes/
│   │       │   ├── mod.rs
│   │       │   ├── health.rs
│   │       │   ├── news.rs
│   │       │   └── openapi.rs
│   │       └── domain/
│   │           └── news.rs
│   └── rust-svc-bad/                # caja INTENCIONALMENTE MALA — escenario 4
│       ├── Cargo.toml               # idéntico a rust-svc
│       ├── Dockerfile               # OCI compliant (pasa conftest)
│       └── src/
│           ├── main.rs              # igual a rust-svc
│           └── routes/
│               ├── mod.rs
│               ├── health.rs        # shape correcto { success, data }
│               ├── news.rs          # ← shape roto + .unwrap() + lógica inline
│               └── openapi.rs
├── conformance/
│   ├── package.json
│   ├── tsconfig.json
│   └── src/
│       ├── cli.ts
│       ├── engine.ts
│       ├── deterministic.ts
│       ├── opencode-agent.ts
│       ├── opus-reflection.ts
│       ├── decision.ts
│       ├── audit.ts
│       └── types.ts
├── policy/
│   ├── conformity-policy.yaml
│   └── rego/
│       ├── oci_artifact.rego
│       ├── oci_runtime.rego
│       ├── oci_metadata.rego
│       └── oci_api.rego
├── caja-manifests/
│   ├── compliant.json               # escenario 1 — verde
│   ├── missing-sbom.json            # escenario 2 — rojo OCI (Step A)
│   ├── missing-labels.json          # escenario 3 — ámbar OCI (Step A)
│   ├── bad-quality.json             # escenario 4 — ámbar calidad (Step B OpenCode)
│   ├── root-user.json               # extra — rojo seguridad
│   └── fixtures/
│       └── sbom-valid.spdx.json
├── audit-log/                    # generado en runtime, gitignore
├── artifacts/                    # generado en CI (SBOM, etc.)
└── .github/
    └── workflows/
        └── conformance-gate.yml
```

---

## FASE 1 — Política de conformidad

### `policy/conformity-policy.yaml`

```yaml
policy_version: "1.0.0"
effective_date: "2026-05-04"
description: "Estándar de caja — cumplimiento requerido antes de merge a main"

rules:
  - id: OCI-001
    name: image_signed
    category: artifact
    severity: critical
    description: Imagen firmada con cosign keyless (Sigstore OIDC)

  - id: OCI-002
    name: sbom_present
    category: artifact
    severity: critical
    description: SBOM en formato SPDX-JSON generado por syft

  - id: OCI-003
    name: healthcheck_defined
    category: runtime
    severity: critical
    description: HEALTHCHECK declarado en Dockerfile

  - id: OCI-004
    name: nonroot_user
    category: security
    severity: high
    description: Imagen corre como usuario no-root

  - id: OCI-005
    name: labels_required
    category: metadata
    severity: high
    required_labels:
      - org.opencontainers.image.version
      - org.opencontainers.image.revision
      - org.opencontainers.image.source
      - com.conformance.owner
      - com.conformance.policy-version
    description: Todos los labels obligatorios presentes y no vacíos

  - id: OCI-006
    name: openapi_spec_accessible
    category: api
    severity: high
    description: GET /openapi.json responde 200 con campo "openapi" en body

  - id: OCI-007
    name: health_endpoint_responding
    category: runtime
    severity: critical
    description: GET /health responde 200 con campo "status"

  - id: BOX-001
    name: response_shape_homogeneous
    category: quality
    severity: high
    description: Todos los endpoints usan el mismo envelope de respuesta (LLM check)

  - id: BOX-002
    name: error_handling_resilient
    category: quality
    severity: high
    description: Manejo de errores sin panics expuestos (LLM check)

  - id: BOX-003
    name: hexagonal_structure
    category: quality
    severity: low
    description: Separación domain/ports/adapters en estructura de módulos (LLM check)
```

---

## FASE 2 — Políticas OPA/Conftest

### `policy/rego/oci_artifact.rego`

```rego
package conformance.artifact

import future.keywords.if
import future.keywords.contains

deny contains msg if {
  not input.sbom_path
  msg := "OCI-002 [critical]: sbom_path es null — SBOM no generado"
}

deny contains msg if {
  input.sbom_path == null
  msg := "OCI-002 [critical]: sbom_path es null — SBOM no generado"
}

deny contains msg if {
  not input.signature_verified
  msg := "OCI-001 [critical]: imagen no firmada con cosign"
}

deny contains msg if {
  input.signature_verified == false
  msg := "OCI-001 [critical]: imagen no firmada con cosign"
}
```

### `policy/rego/oci_runtime.rego`

```rego
package conformance.runtime

import future.keywords.if
import future.keywords.contains

deny contains msg if {
  not input.has_healthcheck
  msg := "OCI-003 [critical]: HEALTHCHECK no definido en la imagen"
}

deny contains msg if {
  input.has_healthcheck == false
  msg := "OCI-003 [critical]: HEALTHCHECK no definido en la imagen"
}

deny contains msg if {
  input.user == ""
  msg := "OCI-004 [high]: USER vacío — la imagen corre como root"
}

deny contains msg if {
  input.user == "0"
  msg := "OCI-004 [high]: USER=0 — la imagen corre como root"
}

deny contains msg if {
  input.user == "root"
  msg := "OCI-004 [high]: USER=root explícito"
}
```

### `policy/rego/oci_metadata.rego`

```rego
package conformance.metadata

import future.keywords.if
import future.keywords.contains

required_labels := {
  "org.opencontainers.image.version",
  "org.opencontainers.image.revision",
  "org.opencontainers.image.source",
  "com.conformance.owner",
  "com.conformance.policy-version"
}

deny contains msg if {
  label := required_labels[_]
  not input.labels[label]
  msg := sprintf("OCI-005 [high]: label obligatorio ausente: %s", [label])
}

deny contains msg if {
  label := required_labels[_]
  input.labels[label] == ""
  msg := sprintf("OCI-005 [high]: label vacío: %s", [label])
}
```

### `policy/rego/oci_api.rego`

```rego
package conformance.api

import future.keywords.if
import future.keywords.contains

warn contains msg if {
  not input.service_url
  msg := "OCI-006 [high]: service_url no disponible — openapi spec no verificada en runtime"
}

warn contains msg if {
  not input.service_url
  msg := "OCI-007 [high]: service_url no disponible — health endpoint no verificado en runtime"
}
```

> Nota: OCI-006 y OCI-007 son `warn` en Rego porque requieren que el contenedor esté corriendo.
> La verificación HTTP real se hace en `deterministic.ts` después de correr conftest.

---

## FASE 3 — Servicio Rust (`services/rust-svc/`)

### `Cargo.toml`

```toml
[package]
name = "rust-svc"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "rust-svc"
path = "src/main.rs"

[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
utoipa = { version = "5", features = ["axum_extras"] }
utoipa-axum = "0.1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
thiserror = "2"
tower-http = { version = "0.6", features = ["cors", "trace"] }
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
reqwest = { version = "0.12", features = ["blocking", "json"] }

[dev-dependencies]
axum-test = "15"
tokio = { version = "1", features = ["full"] }
```

### `src/main.rs`

```rust
use std::sync::Arc;
use axum::Router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod state;
mod error;
mod routes;
mod domain;

use state::AppState;

#[tokio::main]
async fn main() {
    // Subcommand: healthcheck (para el HEALTHCHECK de Docker)
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(|s| s.as_str()) == Some("healthcheck") {
        let res = reqwest::blocking::get("http://localhost:3000/health");
        std::process::exit(if res.map(|r| r.status().is_success()).unwrap_or(false) { 0 } else { 1 });
    }

    // Logging JSON estructurado
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    let state = Arc::new(AppState {
        version: env!("CARGO_PKG_VERSION").to_string(),
    });

    let app = Router::new()
        .merge(routes::health::router())
        .merge(routes::news::router())
        .merge(routes::openapi::router())
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("rust-svc escuchando en 0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}
```

### `src/state.rs`

```rust
#[derive(Clone)]
pub struct AppState {
    pub version: String,
}
```

### `src/error.rs`

```rust
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Recurso no encontrado")]
    NotFound,
    #[error("Error interno: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        (status, Json(json!({ "success": false, "error": message }))).into_response()
    }
}
```

### `src/domain/news.rs`

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct NewsItem {
    pub id: u32,
    pub title: String,
    pub source: String,
    pub published_at: String,
}

pub fn get_top_news() -> Vec<NewsItem> {
    vec![
        NewsItem { id: 1, title: "Rust 2.0 anunciado".into(), source: "blog.rust-lang.org".into(), published_at: "2026-05-04T10:00:00Z".into() },
        NewsItem { id: 2, title: "OpenCode SDK v2 lanzado".into(), source: "opencode.ai".into(), published_at: "2026-05-03T08:00:00Z".into() },
        NewsItem { id: 3, title: "SLSA Level 4 ahora disponible".into(), source: "slsa.dev".into(), published_at: "2026-05-02T12:00:00Z".into() },
    ]
}
```

### `src/routes/health.rs`

```rust
use axum::{extract::State, routing::get, Json, Router};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/health", get(health_handler))
}

/// GET /health
#[utoipa::path(get, path = "/health",
    responses((status = 200, description = "Servicio saludable")))]
async fn health_handler(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "status": "ok",
            "version": state.version,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;

    #[tokio::test]
    async fn health_returns_ok() {
        let state = Arc::new(AppState { version: "test".into() });
        let app = router().with_state(state);
        let server = TestServer::new(app).unwrap();
        let res = server.get("/health").await;
        res.assert_status_ok();
        let body: serde_json::Value = res.json();
        assert_eq!(body["data"]["status"], "ok");
    }
}
```

### `src/routes/news.rs`

```rust
use axum::{extract::State, routing::get, Json, Router};
use serde_json::json;
use std::sync::Arc;
use crate::{domain::news::{get_top_news, NewsItem}, state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/v1/news", get(news_handler))
}

/// GET /api/v1/news
#[utoipa::path(get, path = "/api/v1/news",
    responses((status = 200, body = Vec<NewsItem>, description = "Top noticias")))]
async fn news_handler(State(_state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({ "success": true, "data": get_top_news() }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;

    #[tokio::test]
    async fn news_returns_list() {
        let state = Arc::new(AppState { version: "test".into() });
        let app = router().with_state(state);
        let server = TestServer::new(app).unwrap();
        let res = server.get("/api/v1/news").await;
        res.assert_status_ok();
        let body: serde_json::Value = res.json();
        assert!(body["data"].as_array().unwrap().len() > 0);
    }
}
```

### `src/routes/openapi.rs`

```rust
use axum::{routing::get, Json, Router};
use std::sync::Arc;
use utoipa::OpenApi;
use crate::{domain::news::NewsItem, state::AppState};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::health::health_handler,
        crate::routes::news::news_handler,
    ),
    components(schemas(NewsItem)),
    info(title = "rust-svc", version = "0.1.0")
)]
struct ApiDoc;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/openapi.json", get(openapi_handler))
}

async fn openapi_handler() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
```

### `src/routes/mod.rs`

```rust
pub mod health;
pub mod news;
pub mod openapi;
```

### `src/domain/mod.rs`

```rust
pub mod news;
```

---

## FASE 4 — Dockerfile (`services/rust-svc/Dockerfile`)

```dockerfile
# Stage 1: build
FROM rust:1.87-slim AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
# Cache layer de dependencias
RUN mkdir src && echo "fn main(){}" > src/main.rs && \
    cargo build --release --locked && rm -rf src
COPY src ./src
RUN touch src/main.rs && cargo build --release --locked

# Stage 2: runtime distroless
FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/rust-svc /usr/local/bin/rust-svc

# Labels — se inyectan como build-args en CI
ARG VERSION=dev
ARG GIT_COMMIT=unknown
ARG REPO_URL=unknown
LABEL org.opencontainers.image.version="${VERSION}"
LABEL org.opencontainers.image.revision="${GIT_COMMIT}"
LABEL org.opencontainers.image.source="${REPO_URL}"
LABEL com.conformance.owner="devops-poc"
LABEL com.conformance.policy-version="1.0.0"

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
  CMD ["/usr/local/bin/rust-svc", "healthcheck"]

USER nonroot
EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/rust-svc"]
```

---

## FASE 5 — Motor de Conformidad TypeScript (`conformance/`)

### `conformance/package.json`

```json
{
  "name": "conformance-engine",
  "version": "1.0.0",
  "type": "module",
  "engines": { "node": ">=22" },
  "scripts": {
    "build": "tsc",
    "check": "node dist/cli.js --caja"
  },
  "dependencies": {
    "@opencode-ai/sdk": "latest",
    "@anthropic-ai/sdk": "^0.30.0",
    "zod": "^3.23.0"
  },
  "devDependencies": {
    "typescript": "^5.5.0",
    "@types/node": "^22.0.0"
  }
}
```

### `conformance/tsconfig.json`

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true
  },
  "include": ["src"]
}
```

### `conformance/src/types.ts`

```typescript
export interface CajaManifest {
  image: string;
  sha: string;
  build_time: string;
  repo_url: string;
  sbom_path: string | null;
  signature_verified: boolean;
  labels: Record<string, string>;
  has_healthcheck: boolean;
  user: string;
  service_url: string;
  source_tree_path: string;
}

export interface Violation {
  rule_id: string;
  rule_name: string;
  severity: "critical" | "high" | "low";
  detail: string;
}

export interface Evidence {
  checked_rules: string[];
  passed_rules: string[];
  failed_rules: string[];
  deterministic_findings: Violation[];
  llm_findings: Violation[];
  llm_raw_response?: string;
}

export interface ConformanceDecision {
  run_id: string;
  decision: "pass" | "manual_review" | "block";
  confidence: number;
  policy_version: string;
  evaluated_at: string;
  sha: string;
  image: string;
  violations: Violation[];
  evidence: Evidence;
  suggestions: string[];
  approved_by: string | null;
}
```

### `conformance/src/deterministic.ts`

```typescript
import { execSync } from "child_process";
import { readFileSync } from "fs";
import type { CajaManifest, Violation } from "./types.js";

export function deterministicCheck(
  caja: CajaManifest,
  cajaManifestPath: string,
): { violations: Violation[]; passed: string[] } {
  // Step A1: conftest sobre el manifest JSON (checks estructurales sin runtime)
  let conftestViolations: Violation[] = [];
  let conftestPassed: string[] = [];

  try {
    const output = execSync(
      `conftest test ${cajaManifestPath} --policy ../policy/rego/ --output json 2>&1 || true`,
      { encoding: "utf8" },
    );
    const parsed = JSON.parse(output);
    const failures = parsed[0]?.failures ?? [];
    const successes = parsed[0]?.successes ?? [];

    conftestViolations = failures.map((f: any) => parseConftestMsg(f.message));
    conftestPassed = successes.map((s: any) => extractRuleId(s.message));
  } catch (e) {
    conftestViolations.push({
      rule_id: "CONFTEST",
      rule_name: "conftest_exec",
      severity: "critical",
      detail: `conftest falló: ${e}`,
    });
  }

  // Step A2: checks HTTP de runtime (OCI-006, OCI-007) — requieren servicio corriendo
  const runtimeViolations: Violation[] = [];
  const runtimePassed: string[] = [];

  if (caja.service_url) {
    // OCI-007: /health
    try {
      const res = await fetch(`${caja.service_url}/health`);
      const body = await res.json();
      if (!res.ok || !body?.data?.status) {
        runtimeViolations.push({
          rule_id: "OCI-007",
          rule_name: "health_endpoint_responding",
          severity: "critical",
          detail: `/health respondió ${res.status} sin campo status`,
        });
      } else runtimePassed.push("OCI-007");
    } catch (e) {
      runtimeViolations.push({
        rule_id: "OCI-007",
        rule_name: "health_endpoint_responding",
        severity: "critical",
        detail: `/health no accesible: ${e}`,
      });
    }

    // OCI-006: /openapi.json
    try {
      const res = await fetch(`${caja.service_url}/openapi.json`);
      const body = await res.json();
      if (!res.ok || !body?.openapi) {
        runtimeViolations.push({
          rule_id: "OCI-006",
          rule_name: "openapi_spec_accessible",
          severity: "high",
          detail: `/openapi.json respondió ${res.status} sin campo openapi`,
        });
      } else runtimePassed.push("OCI-006");
    } catch (e) {
      runtimeViolations.push({
        rule_id: "OCI-006",
        rule_name: "openapi_spec_accessible",
        severity: "high",
        detail: `/openapi.json no accesible: ${e}`,
      });
    }
  }

  return {
    violations: [...conftestViolations, ...runtimeViolations],
    passed: [...conftestPassed, ...runtimePassed],
  };
}

function parseConftestMsg(msg: string): Violation {
  const match = msg.match(/^(OCI-\d+|BOX-\d+) \[(critical|high|low)\]: (.+)$/);
  if (match)
    return {
      rule_id: match[1],
      rule_name: match[1],
      severity: match[2] as any,
      detail: match[3],
    };
  return {
    rule_id: "UNKNOWN",
    rule_name: "unknown",
    severity: "high",
    detail: msg,
  };
}

function extractRuleId(msg: string): string {
  return msg.match(/^(OCI-\d+|BOX-\d+)/)?.[1] ?? "UNKNOWN";
}
```

### `conformance/src/opencode-agent.ts`

```typescript
import { createOpencode } from "@opencode-ai/sdk";
import type { CajaManifest, Violation } from "./types.js";

export async function opencodeAgentCheck(
  caja: CajaManifest,
  deterministicViolations: Violation[],
): Promise<{
  findings: Violation[];
  suggestions: string[];
  rawResponse: string;
}> {
  const opencodeUrl = process.env.OPENCODE_SERVER_URL ?? undefined;
  const { client } = await createOpencode(
    opencodeUrl ? { baseUrl: opencodeUrl } : undefined,
  );

  const session = await client.session.create({
    title: `conformance-${caja.sha.substring(0, 8)}`,
  });

  const prompt = `
Eres un evaluador de conformidad de servicios OCI. Analiza la caja y devuelve ÚNICAMENTE JSON sin markdown.

## Caja
- Imagen: ${caja.image}
- SHA: ${caja.sha}
- Source: ${caja.source_tree_path}
- Service URL: ${caja.service_url}

## Violaciones ya detectadas (deterministas — no las repitas)
${JSON.stringify(deterministicViolations, null, 2)}

## Tu tarea: evalúa las reglas BOX (requieren interpretación)

BOX-001 (high): ¿Todos los endpoints del source usan el mismo envelope { success, data }?
  - Inspecciona los archivos en ${caja.source_tree_path}/src/routes/
  - Falla si algún endpoint devuelve un shape diferente

BOX-002 (high): ¿El código maneja errores sin exponer panics ni stack traces?
  - Busca .unwrap() en handlers, ausencia de AppError, panic! expuesto

BOX-003 (low): ¿Hay separación domain/ports/adapters?
  - Busca si la lógica de negocio está en domain/ separada de routes/

## Formato de respuesta (JSON puro, sin bloques de código):
{
  "findings": [
    { "rule_id": "BOX-001", "rule_name": "response_shape_homogeneous", "severity": "high", "detail": "..." }
  ],
  "suggestions": ["..."],
  "analysis_notes": "..."
}
`;

  const result = await client.session.message(session.id, {
    parts: [{ type: "text", text: prompt }],
    model: "anthropic/claude-sonnet-4-6",
  });

  const rawText =
    result.parts?.find((p: any) => p.type === "text")?.text ?? "{}";
  const rawResponse = rawText;

  try {
    const parsed = JSON.parse(rawText);
    return {
      findings: (parsed.findings ?? []) as Violation[],
      suggestions: parsed.suggestions ?? [],
      rawResponse,
    };
  } catch {
    console.error("OpenCode agent devolvió JSON inválido:", rawText);
    return { findings: [], suggestions: [], rawResponse };
  }
}
```

### `conformance/src/opus-reflection.ts`

```typescript
import Anthropic from "@anthropic-ai/sdk";
import type { CajaManifest, Violation } from "./types.js";

export async function opusReflection(
  caja: CajaManifest,
  allViolations: Violation[],
  agentSuggestions: string[],
): Promise<{
  adjusted_confidence: number;
  recommendation: "pass" | "manual_review";
  reasoning: string;
}> {
  const client = new Anthropic();

  const message = await client.messages.create({
    model: "claude-opus-4-6",
    max_tokens: 1024,
    messages: [
      {
        role: "user",
        content: `
Eres el revisor final de un gate de conformidad CI/CD.
La caja tiene violaciones "high" pero ninguna "critical".
Determina si debe hacer pass directamente o requerir manual_review.

Caja: ${caja.image} (${caja.sha})
Repositorio: ${caja.repo_url}

Violaciones encontradas (todas high/low):
${JSON.stringify(allViolations, null, 2)}

Sugerencias del agente:
${agentSuggestions.join("\n")}

Criterio:
- Si las violaciones son cosméticas o de baja criticidad real → pass
- Si alguna implica riesgo de seguridad o contrato roto → manual_review

Responde SOLO en JSON (sin markdown):
{
  "adjusted_confidence": 0.0,
  "recommendation": "pass | manual_review",
  "reasoning": "..."
}
`,
      },
    ],
  });

  const text =
    message.content[0].type === "text" ? message.content[0].text : "{}";
  try {
    return JSON.parse(text);
  } catch {
    return {
      adjusted_confidence: 0.82,
      recommendation: "manual_review",
      reasoning: "JSON parse falló en Opus — fallback conservador",
    };
  }
}
```

### `conformance/src/decision.ts`

```typescript
import type { Violation } from "./types.js";

export function computeDecision(
  deterministicViolations: Violation[],
  agentViolations: Violation[],
  opusResult: { adjusted_confidence: number; recommendation: string } | null,
): { decision: "pass" | "manual_review" | "block"; confidence: number } {
  const all = [...deterministicViolations, ...agentViolations];
  const hasCritical = all.some((v) => v.severity === "critical");
  const hasHigh = all.some((v) => v.severity === "high");

  // Regla dura: critical → block inmediato (Opus no se invoca en este caso)
  if (hasCritical) return { decision: "block", confidence: 1.0 };

  // Sin violaciones → pass limpio
  if (all.length === 0) return { decision: "pass", confidence: 1.0 };

  // Solo low → pass con alta confianza
  if (!hasHigh) return { decision: "pass", confidence: 0.97 };

  // Hay high → usar Opus si está disponible
  if (opusResult) {
    const c = opusResult.adjusted_confidence;
    if (c >= 0.95) return { decision: "pass", confidence: c };
    return { decision: "manual_review", confidence: c };
  }

  // Fallback: high sin Opus → manual_review conservador
  return { decision: "manual_review", confidence: 0.85 };
}
```

### `conformance/src/audit.ts`

```typescript
import { writeFileSync, mkdirSync } from "fs";
import { join } from "path";
import type { ConformanceDecision } from "./types.js";

export function writeAuditLog(decision: ConformanceDecision): void {
  const dir = process.env.AUDIT_LOG_DIR ?? "./audit-log";
  mkdirSync(dir, { recursive: true });
  const ts = decision.evaluated_at.replace(/[:.]/g, "-");
  const sha = decision.sha.substring(0, 8);
  const filename = `${ts}-${sha}.json`;
  writeFileSync(join(dir, filename), JSON.stringify(decision, null, 2), "utf8");
  console.error(`[audit] escrito: ${filename}`);
}
```

### `conformance/src/engine.ts`

```typescript
import { randomUUID } from "crypto";
import type { CajaManifest, ConformanceDecision } from "./types.js";
import { deterministicCheck } from "./deterministic.js";
import { opencodeAgentCheck } from "./opencode-agent.js";
import { opusReflection } from "./opus-reflection.js";
import { computeDecision } from "./decision.js";
import { writeAuditLog } from "./audit.js";

export async function evaluateCaja(
  caja: CajaManifest,
  cajaManifestPath: string,
): Promise<ConformanceDecision> {
  const run_id = randomUUID();

  // Step A — OPA/Conftest + HTTP checks (sin LLM)
  const { violations: detViolations, passed: detPassed } = deterministicCheck(
    caja,
    cajaManifestPath,
  );

  // Cortocircuito: critical → block, no gastar LLM
  const hasCriticalDet = detViolations.some((v) => v.severity === "critical");
  if (hasCriticalDet) {
    const result: ConformanceDecision = {
      run_id,
      decision: "block",
      confidence: 1.0,
      policy_version: "1.0.0",
      evaluated_at: new Date().toISOString(),
      sha: caja.sha,
      image: caja.image,
      violations: detViolations,
      evidence: {
        checked_rules: [...detPassed, ...detViolations.map((v) => v.rule_id)],
        passed_rules: detPassed,
        failed_rules: detViolations.map((v) => v.rule_id),
        deterministic_findings: detViolations,
        llm_findings: [],
      },
      suggestions: ["Corregir las violaciones critical antes de re-evaluar"],
      approved_by: null,
    };
    writeAuditLog(result);
    return result;
  }

  // Step B — OpenCode agent (checks cualitativos BOX-001/002/003)
  const {
    findings: agentFindings,
    suggestions,
    rawResponse,
  } = await opencodeAgentCheck(caja, detViolations);

  // Step C — Opus reflection SOLO si hay high violations
  const hasHigh = [...detViolations, ...agentFindings].some(
    (v) => v.severity === "high",
  );
  let opusResult = null;
  if (hasHigh) {
    opusResult = await opusReflection(
      caja,
      [...detViolations, ...agentFindings],
      suggestions,
    );
  }

  const { decision, confidence } = computeDecision(
    detViolations,
    agentFindings,
    opusResult,
  );

  const result: ConformanceDecision = {
    run_id,
    decision,
    confidence,
    policy_version: "1.0.0",
    evaluated_at: new Date().toISOString(),
    sha: caja.sha,
    image: caja.image,
    violations: [...detViolations, ...agentFindings],
    evidence: {
      checked_rules: [
        ...detPassed,
        ...detViolations.map((v) => v.rule_id),
        ...agentFindings.map((v) => v.rule_id),
      ],
      passed_rules: detPassed,
      failed_rules: [...detViolations, ...agentFindings].map((v) => v.rule_id),
      deterministic_findings: detViolations,
      llm_findings: agentFindings,
      llm_raw_response: rawResponse,
    },
    suggestions,
    approved_by: null,
  };

  writeAuditLog(result);
  return result;
}
```

### `conformance/src/cli.ts`

```typescript
import { readFileSync } from "fs";
import { evaluateCaja } from "./engine.js";
import type { CajaManifest } from "./types.js";

const args = process.argv.slice(2);
const cajaIdx = args.indexOf("--caja");
if (cajaIdx === -1 || !args[cajaIdx + 1]) {
  console.error("Uso: node dist/cli.js --caja <path-to-caja-manifest.json>");
  process.exit(1);
}

const cajaManifestPath = args[cajaIdx + 1];
const caja: CajaManifest = JSON.parse(readFileSync(cajaManifestPath, "utf8"));

try {
  const decision = await evaluateCaja(caja, cajaManifestPath);
  // stdout: solo el JSON de decisión (para que CI lo parsee)
  console.log(JSON.stringify(decision, null, 2));
  process.exit(decision.decision === "block" ? 1 : 0);
} catch (e) {
  console.error("Error fatal en conformance engine:", e);
  process.exit(2);
}
```

---

## FASE 6 — Servicio Rust malo (`services/rust-svc-bad/`)

**Propósito:** caja que pasa TODOS los checks OCI/Conftest (Step A) pero viola reglas de calidad BOX (Step B). Demuestra que OpenCode SDK agrega valor real más allá de conftest.

**Las violaciones intencionadas:**

| Regla   | Violación introducida                                                                     |
| ------- | ----------------------------------------------------------------------------------------- |
| BOX-001 | `/news` devuelve array crudo sin wrapper; `/health` usa `{success, data}` — inconsistente |
| BOX-002 | `.unwrap()` en handler de news — pánico potencial en producción                           |
| BOX-003 | Sin directorio `domain/` — lógica de negocio inline dentro del handler                    |

### `services/rust-svc-bad/Cargo.toml`

Idéntico a `rust-svc/Cargo.toml`. Mismo nombre de paquete, mismas dependencias.

### `services/rust-svc-bad/Dockerfile`

Idéntico a `rust-svc/Dockerfile` — OCI completamente conforme:

- Multi-stage distroless
- Labels completos
- HEALTHCHECK declarado
- USER nonroot

### `services/rust-svc-bad/src/main.rs`

Idéntico a `rust-svc/src/main.rs` — el arranque es correcto. Los problemas están en las rutas.

### `services/rust-svc-bad/src/routes/health.rs` — shape CORRECTO (control)

```rust
// Este endpoint SÍ usa el wrapper — para hacer el contraste visible a OpenCode
use axum::{extract::State, routing::get, Json, Router};
use serde_json::{json, Value};
use std::sync::Arc;
use crate::state::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/health", get(health_handler))
}

async fn health_handler(State(state): State<Arc<AppState>>) -> Json<Value> {
    Json(json!({
        "success": true,
        "data": {
            "status": "ok",
            "version": state.version,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    }))
}
```

### `services/rust-svc-bad/src/routes/news.rs` — VIOLACIONES INTENCIONALES

```rust
// BOX-001: respuesta SIN wrapper { success, data } — array directo
// BOX-002: .unwrap() en handler — panic potencial
// BOX-003: lógica de negocio inline, sin domain/
use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::state::AppState;

#[derive(Serialize, Deserialize)]
struct NewsItem {
    id: u32,
    title: String,
    source: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/api/v1/news", get(news_handler))
}

// BOX-001: devuelve Vec<NewsItem> directo — sin { success: true, data: [...] }
// BOX-002: parse_count().unwrap() — pánico si la variable de entorno no es número
// BOX-003: toda la lógica de construcción de noticias está aquí mismo
async fn news_handler() -> Json<Vec<NewsItem>> {
    // Lógica de negocio inline — viola HEX (debería estar en domain/)
    let count: usize = std::env::var("NEWS_COUNT")
        .unwrap_or("3".to_string())
        .parse()
        .unwrap(); // BOX-002: pánico si NEWS_COUNT="abc"

    let items: Vec<NewsItem> = (1..=count)
        .map(|i| NewsItem {
            id: i as u32,
            title: format!("Noticia {}", i),
            source: "inline-hardcoded".to_string(), // BOX-003: sin abstracción
        })
        .collect();

    Json(items) // BOX-001: respuesta sin envelope { success, data }
}
```

### `services/rust-svc-bad/src/routes/openapi.rs`

Idéntico al de `rust-svc` — expone `/openapi.json` correctamente (OCI-006 pasa).

---

## FASE 7 — Manifests de prueba

### `caja-manifests/compliant.json` → escenario VERDE (pass)

```json
{
  "image": "ghcr.io/test/rust-svc:abc1234",
  "sha": "abc1234def567890",
  "build_time": "2026-05-04T10:00:00Z",
  "repo_url": "https://github.com/test/conformance-gate-poc",
  "sbom_path": "./caja-manifests/fixtures/sbom-valid.spdx.json",
  "signature_verified": true,
  "labels": {
    "org.opencontainers.image.version": "0.1.0",
    "org.opencontainers.image.revision": "abc1234",
    "org.opencontainers.image.source": "https://github.com/test/conformance-gate-poc",
    "com.conformance.owner": "devops-poc",
    "com.conformance.policy-version": "1.0.0"
  },
  "has_healthcheck": true,
  "user": "nonroot",
  "service_url": "http://localhost:3000",
  "source_tree_path": "./services/rust-svc"
}
```

### `caja-manifests/missing-sbom.json` → escenario ROJO (block)

```json
{
  "image": "ghcr.io/test/rust-svc:nosbom",
  "sha": "nosbom000000000",
  "build_time": "2026-05-04T10:00:00Z",
  "repo_url": "https://github.com/test/conformance-gate-poc",
  "sbom_path": null,
  "signature_verified": false,
  "labels": {
    "org.opencontainers.image.version": "0.1.0",
    "org.opencontainers.image.revision": "nosbom0",
    "org.opencontainers.image.source": "https://github.com/test/conformance-gate-poc",
    "com.conformance.owner": "devops-poc",
    "com.conformance.policy-version": "1.0.0"
  },
  "has_healthcheck": true,
  "user": "nonroot",
  "service_url": "http://localhost:3000",
  "source_tree_path": "./services/rust-svc"
}
```

### `caja-manifests/missing-labels.json` → escenario ÁMBAR (manual_review)

```json
{
  "image": "ghcr.io/test/rust-svc:nolabels",
  "sha": "nolabels0000000",
  "build_time": "2026-05-04T10:00:00Z",
  "repo_url": "https://github.com/test/conformance-gate-poc",
  "sbom_path": "./caja-manifests/fixtures/sbom-valid.spdx.json",
  "signature_verified": true,
  "labels": {
    "org.opencontainers.image.version": "0.1.0"
  },
  "has_healthcheck": true,
  "user": "nonroot",
  "service_url": "http://localhost:3000",
  "source_tree_path": "./services/rust-svc"
}
```

### `caja-manifests/bad-quality.json` → escenario CALIDAD MALA (Step B OpenCode actúa)

**OCI: conforme. Calidad: no conforme.** Pasa conftest, falla en BOX rules que detecta OpenCode.

```json
{
  "image": "ghcr.io/test/rust-svc-bad:quality001",
  "sha": "quality001abc000",
  "build_time": "2026-05-04T10:00:00Z",
  "repo_url": "https://github.com/test/conformance-gate-poc",
  "sbom_path": "./caja-manifests/fixtures/sbom-valid.spdx.json",
  "signature_verified": true,
  "labels": {
    "org.opencontainers.image.version": "0.1.0",
    "org.opencontainers.image.revision": "quality001",
    "org.opencontainers.image.source": "https://github.com/test/conformance-gate-poc",
    "com.conformance.owner": "devops-poc",
    "com.conformance.policy-version": "1.0.0"
  },
  "has_healthcheck": true,
  "user": "nonroot",
  "service_url": "http://localhost:3001",
  "source_tree_path": "./services/rust-svc-bad"
}
```

> Nota: `service_url` apunta a `localhost:3001` — el servicio malo corre en puerto diferente para no colisionar con rust-svc en pruebas locales.

### `caja-manifests/fixtures/sbom-valid.spdx.json`

```json
{
  "SPDXID": "SPDXRef-DOCUMENT",
  "spdxVersion": "SPDX-2.3",
  "creationInfo": {
    "created": "2026-05-04T10:00:00Z",
    "creators": ["Tool: syft"]
  },
  "name": "rust-svc",
  "dataLicense": "CC0-1.0",
  "documentNamespace": "https://github.com/test/conformance-gate-poc/sbom",
  "packages": []
}
```

---

## FASE 8 — GitHub Actions Workflow

### `.github/workflows/conformance-gate.yml`

```yaml
name: Conformance Gate

on:
  pull_request:
    branches: [main]

permissions:
  contents: read
  id-token: write # cosign keyless necesita OIDC token
  pull-requests: write

env:
  IMAGE: ghcr.io/${{ github.repository }}/rust-svc:${{ github.sha }}

jobs:
  # ──────────────────────────────────────────────────────────
  # JOB 1 — Build + tests unitarios (compilación + correctitud)
  # ──────────────────────────────────────────────────────────
  build-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: cargo test + clippy
        run: |
          cd services/rust-svc
          cargo clippy -- -D warnings
          cargo test

  # ──────────────────────────────────────────────────────────
  # JOB 2 — SAST (análisis estático de seguridad)
  # Corre DESPUÉS de build-test y ANTES del Conformance Gate.
  #
  # NOTA DE DISEÑO — por qué no está SonarQube/SonarCloud:
  # SonarQube (y cualquier herramienta estática) NO detectaría BOX-001/002/003
  # en rust-svc-bad. El código compila, no tiene CVEs, no tiene code smells
  # mecánicos — pero viola contratos semánticos y arquitecturales.
  # Agregar SonarQube no fortalece ni debilita el PoC porque no cambia ese hecho.
  # La propuesta de valor del agente es precisamente lo que ninguna herramienta
  # estática puede ver. Las herramientas incluidas son:
  #   - cargo clippy → linter Rust (ya en JOB 1, dominio: estilo y antipatrones)
  #   - cargo audit  → CVEs en dependencias (dominio: seguridad de supply chain)
  # Ambas pasarán rust-svc-bad sin encontrar nada. OpenCode no.
  # ──────────────────────────────────────────────────────────
  sast:
    needs: build-test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # cargo-audit: busca CVEs en Cargo.lock contra la RustSec Advisory DB
      # Equivalente a `npm audit` — dominio diferente al del agente OpenCode
      - name: Install cargo-audit
        run: cargo install cargo-audit --locked

      - name: cargo audit (CVE scan rust-svc)
        run: |
          cd services/rust-svc
          cargo audit

      - name: cargo audit (CVE scan rust-svc-bad)
        run: |
          cd services/rust-svc-bad
          cargo audit

  # ──────────────────────────────────────────────────────────
  # JOB 3 — Conformance Gate (determinista + LLM)
  # Solo corre si build-test Y sast pasan
  # ──────────────────────────────────────────────────────────
  conformance:
    needs: [build-test, sast]
    runs-on: ubuntu-latest
    environment: staging # required_reviewers para manual_review
    env:
      ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}

    steps:
      - uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Build imagen OCI con labels
        run: |
          docker build \
            --build-arg VERSION=0.1.0 \
            --build-arg GIT_COMMIT=${{ github.sha }} \
            --build-arg REPO_URL=${{ github.server_url }}/${{ github.repository }} \
            -t $IMAGE ./services/rust-svc

      - name: Install conftest
        run: |
          curl -sL https://github.com/open-policy-agent/conftest/releases/latest/download/conftest_Linux_x86_64.tar.gz | tar xz
          sudo mv conftest /usr/local/bin/

      - name: Install syft
        uses: anchore/sbom-action/download-syft@v0

      - name: Generate SBOM
        run: |
          mkdir -p artifacts
          syft image $IMAGE -o spdx-json > artifacts/sbom.spdx.json

      - name: Install cosign
        uses: sigstore/cosign-installer@v3

      - name: Sign image keyless
        run: cosign sign --yes $IMAGE

      - name: Start rust-svc para runtime checks
        run: |
          docker run -d --name rust-svc-test -p 3000:3000 $IMAGE
          for i in {1..15}; do
            curl -sf http://localhost:3000/health && break || sleep 2
          done

      - name: Install OpenCode
        run: npm install -g opencode@latest

      - name: Start OpenCode sidecar
        run: opencode serve --port 4096 &
        env:
          OPENCODE_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}

      - name: Generate caja-manifest.json
        run: |
          mkdir -p artifacts
          LABELS=$(docker inspect $IMAGE --format='{{json .Config.Labels}}')
          HAS_HC=$(docker inspect $IMAGE --format='{{if .Config.Healthcheck}}true{{else}}false{{end}}')
          USER_VAL=$(docker inspect $IMAGE --format='{{.Config.User}}')
          cat > artifacts/caja-manifest.json <<EOF
          {
            "image": "$IMAGE",
            "sha": "${{ github.sha }}",
            "build_time": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
            "repo_url": "${{ github.server_url }}/${{ github.repository }}",
            "sbom_path": "${{ github.workspace }}/artifacts/sbom.spdx.json",
            "signature_verified": true,
            "labels": $LABELS,
            "has_healthcheck": $HAS_HC,
            "user": "$USER_VAL",
            "service_url": "http://localhost:3000",
            "source_tree_path": "${{ github.workspace }}/services/rust-svc"
          }
          EOF

      - name: Setup Node 22
        uses: actions/setup-node@v4
        with:
          node-version: 22

      - name: Build conformance engine
        run: cd conformance && npm ci && npm run build

      - name: Run Conformance Gate
        id: gate
        env:
          AUDIT_LOG_DIR: ${{ github.workspace }}/audit-log
          OPENCODE_SERVER_URL: http://localhost:4096
        run: |
          set +e
          RESULT=$(node conformance/dist/cli.js --caja artifacts/caja-manifest.json)
          EXIT_CODE=$?
          set -e
          echo "$RESULT" > artifacts/conformance-result.json
          DECISION=$(echo "$RESULT" | jq -r .decision)
          CONFIDENCE=$(echo "$RESULT" | jq -r .confidence)
          echo "decision=$DECISION" >> $GITHUB_OUTPUT
          echo "confidence=$CONFIDENCE" >> $GITHUB_OUTPUT

      - name: Upload artifacts
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: conformance-${{ github.sha }}
          path: |
            artifacts/
            audit-log/

      - name: Comentar en PR
        if: always()
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          DECISION="${{ steps.gate.outputs.decision }}"
          CONFIDENCE="${{ steps.gate.outputs.confidence }}"
          VIOLATIONS=$(cat artifacts/conformance-result.json | jq '.violations | length')
          ICON="✅"
          [ "$DECISION" = "manual_review" ] && ICON="⚠️"
          [ "$DECISION" = "block" ] && ICON="❌"
          gh pr comment ${{ github.event.pull_request.number }} --body \
            "$ICON **Conformance Gate: \`$DECISION\`**
            - Confianza: \`$CONFIDENCE\`
            - Violaciones: \`$VIOLATIONS\`
            - Artifacts: ver pestaña Summary del workflow"

      - name: Evaluar decisión final
        run: |
          DECISION="${{ steps.gate.outputs.decision }}"
          if [ "$DECISION" = "block" ]; then
            echo "❌ Pipeline bloqueado por Conformance Gate"
            exit 1
          fi
          if [ "$DECISION" = "manual_review" ]; then
            echo "⚠️ Requiere aprobación en GitHub Environment 'staging'"
            # El job ya está en environment 'staging' — GitHub pausa aquí
            # hasta que un reviewer apruebe o expire el wait timer
          fi
          echo "✅ Conformance Gate aprobado"
```

**Configuración GitHub requerida (una sola vez):**

```
Settings → Environments → staging:
  - Required reviewers: [tu usuario]
  - Wait timer: 48 horas
Settings → Secrets:
  - ANTHROPIC_API_KEY: tu clave de API
```

---

## VARIABLES DE ENTORNO

| Variable              | Quién la usa                            | Descripción                                        |
| --------------------- | --------------------------------------- | -------------------------------------------------- |
| `ANTHROPIC_API_KEY`   | `opus-reflection.ts` + OpenCode sidecar | Clave Anthropic                                    |
| `OPENCODE_SERVER_URL` | `opencode-agent.ts`                     | URL del sidecar (default: `http://localhost:4096`) |
| `AUDIT_LOG_DIR`       | `audit.ts`                              | Directorio de salida (default: `./audit-log`)      |

---

## TABLA DE ESCENARIOS (5 en total)

| #   | Nombre           | Manifest               | Step que actúa        | Decisión esperada      | Qué demuestra                                              |
| --- | ---------------- | ---------------------- | --------------------- | ---------------------- | ---------------------------------------------------------- |
| 1   | Verde            | `compliant.json`       | ninguno falla         | `pass` confidence=1.0  | No bloquea trabajo válido                                  |
| 2   | Rojo OCI         | `missing-sbom.json`    | Step A (conftest)     | `block` confidence=1.0 | Gate detecta críticos sin LLM, cortocircuito inmediato     |
| 3   | Ámbar OCI        | `missing-labels.json`  | Step A (conftest)     | `manual_review`        | Gate escala a humano en grises de metadatos                |
| 4   | **Calidad mala** | **`bad-quality.json`** | **Step B (OpenCode)** | **`manual_review`**    | **OpenCode detecta BOX-001/002/003 que conftest no puede** |
| 5   | Regresión x5     | `compliant.json` ×5    | ninguno               | `pass` ×5              | Resultado estable, no hay drift entre corridas             |

**El escenario 4 es el más importante para el PoC**: es el único donde OpenCode demuestra valor real. Sin él, el sistema es solo conftest + CI, lo cual ya existe.

---

## VERIFICACIÓN DE LOS 5 ESCENARIOS

```bash
# ─── Prerequisitos ───────────────────────────────────────────────────
# rust-svc (escenarios 1, 2, 3, 5) en puerto 3000
cd services/rust-svc && cargo run &

# rust-svc-bad (escenario 4) en puerto 3001
PORT=3001 cd services/rust-svc-bad && cargo run &

# Compilar motor
cd conformance && npm ci && npm run build

# ─── Escenario 1: VERDE ── pass ───────────────────────────────────────
node conformance/dist/cli.js --caja caja-manifests/compliant.json | jq '{decision, confidence, violations: (.violations | length)}'
# Esperado: { "decision": "pass", "confidence": 1.0, "violations": 0 }

# ─── Escenario 2: ROJO OCI ── block ──────────────────────────────────
node conformance/dist/cli.js --caja caja-manifests/missing-sbom.json | jq '{decision, violations: [.violations[].rule_id]}'
# Esperado: { "decision": "block", "violations": ["OCI-001","OCI-002"] }
# Verificar que OpenCode NO fue llamado (cortocircuito en Step A)
# → audit log debe tener llm_findings: [] y llm_raw_response ausente

# ─── Escenario 3: ÁMBAR OCI ── manual_review ─────────────────────────
node conformance/dist/cli.js --caja caja-manifests/missing-labels.json | jq '{decision, confidence, violations: [.violations[].rule_id]}'
# Esperado: { "decision": "manual_review", violations: ["OCI-005",...] }

# ─── Escenario 4: CALIDAD MALA ── manual_review (OpenCode actúa) ─────
node conformance/dist/cli.js --caja caja-manifests/bad-quality.json | jq '{decision, confidence, det: [.evidence.deterministic_findings[].rule_id], llm: [.evidence.llm_findings[].rule_id]}'
# Esperado:
# {
#   "decision": "manual_review",
#   "confidence": ~0.84,
#   "det": [],             ← conftest no encontró nada (OCI compliant)
#   "llm": ["BOX-001","BOX-002","BOX-003"]  ← OpenCode encontró las violaciones
# }
# Verificar que el audit log tiene llm_raw_response con el análisis de OpenCode

# ─── Escenario 5: REGRESIÓN ── mismo resultado 5 veces ───────────────
for i in {1..5}; do
  RESULT=$(node conformance/dist/cli.js --caja caja-manifests/compliant.json)
  echo "Corrida $i: $(echo $RESULT | jq -r '{decision, confidence}')"
done
# Esperado: "pass" 1.0 en todas las corridas

# ─── Verificación del audit log ──────────────────────────────────────
ls -la audit-log/
cat audit-log/$(ls audit-log/ | tail -1) | jq '{run_id, decision, sha, evidence: {det: (.evidence.deterministic_findings | length), llm: (.evidence.llm_findings | length)}}'
```

---

## CHECKLIST FINAL DE COMPLETITUD

| Criterio                     | Cómo verificar                                                                                         |
| ---------------------------- | ------------------------------------------------------------------------------------------------------ |
| ≥3 violaciones detectadas    | escenario 2 muestra OCI-001 + OCI-002; escenario 4 muestra BOX-001 + BOX-002 + BOX-003                 |
| Bloqueo real en CI           | PR con missing-sbom → check falla, merge no permitido                                                  |
| Manual review funcional      | PR con missing-labels → job pausado en Environment staging                                             |
| **OpenCode demuestra valor** | **escenario 4: `det: []`, `llm: ["BOX-001","BOX-002","BOX-003"]` — conftest no vio nada, OpenCode sí** |
| Cortocircuito en critical    | escenario 2: `llm_findings: []` — OpenCode nunca llamado cuando hay critical                           |
| Audit log trazable           | `cat audit-log/*.json` → run_id, sha, decision, evidence completos                                     |
| Regresión estable            | 5 corridas verde → siempre pass, confidence = 1.0                                                      |
| OpenCode como runtime        | `opencode-agent.ts` usa `@opencode-ai/sdk`, no `@anthropic-ai/sdk`                                     |
| Opus solo en ambiguos        | `engine.ts` solo llama `opusReflection` cuando `!hasCritical && hasHigh`                               |
| Conftest independiente       | `conftest test caja-manifest.json --policy policy/rego/` funciona sin Node                             |
