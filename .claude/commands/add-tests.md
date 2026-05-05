# Skill: add-tests

Agrega tests a un archivo de routes existente siguiendo las reglas de CLAUDE.md.

## Uso

```
/add-tests <PATH_AL_ARCHIVO>
```

Ejemplo: `/add-tests services/rust-svc/src/routes/news.rs`

---

## Tests obligatorios por endpoint

Para cada handler en el archivo, crea estos tests dentro del bloque `#[cfg(test)]`:

### Test 1 — Status HTTP

```rust
#[tokio::test]
async fn <nombre>_returns_ok() {
    let state = Arc::new(AppState { version: "test".into() });
    let app = router().with_state(state);
    let server = TestServer::new(app).unwrap();
    let res = server.get("<PATH>").await;
    res.assert_status_ok();
}
```

### Test 2 — Shape del envelope (BOX-001)

```rust
#[tokio::test]
async fn <nombre>_returns_envelope() {
    let state = Arc::new(AppState { version: "test".into() });
    let app = router().with_state(state);
    let server = TestServer::new(app).unwrap();
    let res = server.get("<PATH>").await;
    let body: serde_json::Value = res.json();
    assert_eq!(body["success"], true);
    assert!(!body["data"].is_null());
}
```

### Test 3 — Respuesta de error (si el endpoint puede fallar)

```rust
#[tokio::test]
async fn <nombre>_not_found_returns_error_envelope() {
    let state = Arc::new(AppState { version: "test".into() });
    let app = router().with_state(state);
    let server = TestServer::new(app).unwrap();
    let res = server.get("<PATH>/id-que-no-existe").await;
    res.assert_status(StatusCode::NOT_FOUND);
    let body: serde_json::Value = res.json();
    assert_eq!(body["success"], false);
    assert!(body["error"].is_string());
}
```

---

## Imports necesarios en el bloque de tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use std::sync::Arc;
    use crate::state::AppState;
    // si necesitas verificar status code concretos:
    // use axum::http::StatusCode;
}
```

---

## Reglas

- Los tests van **dentro del mismo archivo** del route, en `#[cfg(test)]`
- Nunca uses `.unwrap()` en el código del handler, sí puedes usarlo en tests
- Verifica siempre el **status HTTP** y el **shape `{ success, data }`**
- Si el endpoint recibe parámetros, agrega un test con parámetros válidos y otro con inválidos

---

## Checklist

- [ ] Test de status HTTP 200 para el camino feliz
- [ ] Test de shape: `success: true` + `data` presente
- [ ] Test de error si aplica: `success: false` + `error` string
- [ ] Todos los tests usan `axum_test::TestServer`
- [ ] No hay `.unwrap()` en el handler (solo en los tests si es necesario)
