# Skill: create-endpoint

Crea un nuevo endpoint en el servicio Rust siguiendo las reglas de CLAUDE.md.

## Uso

```
/create-endpoint <MÉTODO> <PATH> <DESCRIPCIÓN>
```

Ejemplo: `/create-endpoint GET /api/v1/users Devuelve lista de usuarios`

---

## Pasos que debes seguir

### 1. Crear el modelo en `domain/`

Crea `src/domain/<recurso>.rs` con:

- El struct del recurso con `#[derive(Serialize, Deserialize, ToSchema, Clone)]`
- La función que devuelve los datos (puede ser mock por ahora)
- Expón el módulo en `src/domain/mod.rs`

```rust
// src/domain/<recurso>.rs
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct <Recurso> {
    pub id: u32,
    // campos del recurso...
}

pub fn get_<recursos>() -> Vec<<Recurso>> {
    vec![
        // datos de ejemplo
    ]
}
```

### 2. Crear el handler en `routes/`

Crea `src/routes/<recurso>.rs` con:

- La función `router()` que registra la ruta
- El handler usando el envelope `{ success, data }`
- Anotación `#[utoipa::path(...)]`
- Un bloque `#[cfg(test)]` con al menos 2 tests

```rust
// src/routes/<recurso>.rs
use axum::{extract::State, routing::get, Json, Router};
use serde_json::json;
use std::sync::Arc;
use crate::{domain::<recurso>::{get_<recursos>, <Recurso>}, state::AppState};

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("<PATH>", get(<recurso>_handler))
}

#[utoipa::path(get, path = "<PATH>",
    responses((status = 200, body = Vec<<Recurso>>, description = "<DESCRIPCIÓN>")))]
async fn <recurso>_handler(State(_state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({ "success": true, "data": get_<recursos>() }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;

    #[tokio::test]
    async fn <recurso>_returns_ok() {
        let state = Arc::new(AppState { version: "test".into() });
        let app = router().with_state(state);
        let server = TestServer::new(app).unwrap();
        let res = server.get("<PATH>").await;
        res.assert_status_ok();
    }

    #[tokio::test]
    async fn <recurso>_returns_envelope() {
        let state = Arc::new(AppState { version: "test".into() });
        let app = router().with_state(state);
        let server = TestServer::new(app).unwrap();
        let res = server.get("<PATH>").await;
        let body: serde_json::Value = res.json();
        assert_eq!(body["success"], true);
        assert!(body["data"].is_array());
    }
}
```

### 3. Registrar en `routes/mod.rs`

```rust
pub mod <recurso>;
```

### 4. Conectar en `main.rs`

```rust
.merge(routes::<recurso>::router())
```

### 5. Agregar a la spec OpenAPI en `routes/openapi.rs`

```rust
paths(
    // ... rutas existentes ...
    crate::routes::<recurso>::<recurso>_handler,
),
components(schemas(<Recurso>)),
```

---

## Checklist antes de terminar

- [ ] El handler NO contiene lógica de negocio (está en `domain/`)
- [ ] Respuesta usa `{ "success": true, "data": ... }` o `{ "success": false, "error": ... }`
- [ ] No hay `.unwrap()` ni `.expect()` fuera de tests
- [ ] Hay al menos 2 tests: status OK + shape del envelope
- [ ] El path está registrado en el router de `main.rs`
- [ ] El modelo está en la spec OpenAPI
