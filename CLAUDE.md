# CLAUDE.md — Reglas para construir APIs Rust en este proyecto

Todo código que generes en `services/` debe cumplir estas reglas.
Son las mismas que el agente autónomo de CI va a validar.

---

## Stack obligatorio

- **Rust edition 2021**
- **axum 0.8** — router HTTP
- **tokio** — runtime async
- **utoipa 5** — generación de OpenAPI
- **utoipa-swagger-ui 8** — Swagger UI interactiva
- **serde / serde_json** — serialización
- **thiserror** — errores tipados
- **tracing + tracing-subscriber** — logs estructurados JSON
- **axum-test** — tests de integración (dev-dependency)

---

## Estructura de carpetas (obligatoria)

```
services/<nombre-svc>/
├── Cargo.toml
├── Dockerfile
└── src/
    ├── main.rs          ← arranque del servidor, sin lógica de negocio
    ├── state.rs         ← AppState compartido entre handlers
    ├── error.rs         ← AppError con impl IntoResponse
    ├── domain/
    │   ├── mod.rs
    │   └── <entidad>.rs ← modelos de datos y lógica de dominio
    └── routes/
        ├── mod.rs
        ├── health.rs    ← GET /health  (siempre presente)
        ├── openapi.rs   ← GET /openapi.json  (siempre presente)
        └── <recurso>.rs ← un archivo por recurso
```

**Regla:** La lógica de negocio va en `domain/`. Los handlers en `routes/` solo orquestan.

---

## BOX-001 — Envelope de respuesta homogéneo (obligatorio)

Todos los endpoints deben devolver el mismo shape. Sin excepciones.

**Éxito:**
```json
{ "success": true, "data": <payload> }
```

**Error:**
```json
{ "success": false, "error": "<mensaje legible>" }
```

Implementa esto en `error.rs` con `IntoResponse` y úsalo en todos los handlers.

---

## BOX-002 — Sin panics expuestos (obligatorio)

- **Prohibido:** `.unwrap()`, `.expect()`, `panic!()` en código de producción
- **Obligatorio:** propagar errores con `?` y tipos `Result<T, AppError>`
- Los errores se mapean a `AppError::NotFound`, `AppError::Internal`, etc.

---

## BOX-003 — Separación domain / routes (obligatorio)

- `routes/` contiene handlers HTTP y nada más
- `domain/` contiene modelos, structs y funciones de negocio
- Los handlers importan desde `domain/`, nunca al revés

---

## BOX-004 — Swagger UI accesible (obligatorio)

Monta la UI interactiva para probar endpoints en local sin Postman.

- Dependencia: `utoipa-swagger-ui = { version = "8", features = ["axum"] }`
- Monta en `main.rs`:

```rust
use utoipa_swagger_ui::SwaggerUi;
use routes::openapi::ApiDoc;

// Dentro del Router::new():
.merge(SwaggerUi::new("/swagger-ui").url("/openapi.json", ApiDoc::openapi()))
```

- `ApiDoc` debe ser `pub` en `routes/openapi.rs`

---

## OCI-006 / OCI-007 — Endpoints obligatorios

Todo servicio debe tener estos endpoints desde el día uno:

```
GET /health        → { "success": true, "data": { "status": "ok", "version": "..." } }
GET /openapi.json  → spec OpenAPI generada por utoipa
GET /swagger-ui/   → UI interactiva (BOX-004)
```

---

## Tests (obligatorios en cada route)

Cada archivo en `routes/` debe tener un bloque `#[cfg(test)]` con al menos:
- Un test que valide el status HTTP
- Un test que valide el shape de la respuesta

Usa `axum-test::TestServer`. Ver skill: `.claude/sk/add-tests.md`

---

## Dockerfile (obligatorio)

Reglas no negociables:
1. **Multi-stage** — stage `builder` (rust:slim) + stage `runtime` (distroless)
2. **USER nonroot** — nunca correr como root
3. **HEALTHCHECK** definido con el binario propio
4. **Labels OCI** inyectados como `ARG` en build:
   - `org.opencontainers.image.version`
   - `org.opencontainers.image.revision`
   - `org.opencontainers.image.source`
   - `com.conformance.owner`
   - `com.conformance.policy-version`

---

## Skills disponibles

Usa estas skills para tareas comunes:

| Tarea | Skill |
|-------|-------|
| Crear un nuevo endpoint | `.claude/commands/create-endpoint.md` |
| Agregar tests a un route | `.claude/commands/add-tests.md` |
| Patterns utoipa + axum | `.claude/commands/utoipa-axum.md` |

---

## Lo que el agente autónomo de CI va a validar

Al hacer push, el agente revisa:

1. ¿Todos los endpoints usan `{ success, data }`?
2. ¿Hay algún `.unwrap()` en código de producción?
3. ¿La lógica de negocio está en `domain/` y no en `routes/`?
4. ¿Existen `/health`, `/openapi.json` y `/swagger-ui/`?
5. ¿El Dockerfile tiene HEALTHCHECK, USER nonroot y labels?

Si algo falla → el PR se bloquea.
