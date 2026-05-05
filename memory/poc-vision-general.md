# Hoja de Ruta Conceptual — PoC Conformance Gate

## Qué estamos construyendo y por qué

> Documento de contexto — para entender el sistema completo antes de leer el plan técnico.
> Basado en el diagrama de arquitectura del ecosistema.

---

## La idea central en una frase

> Un agente IA verifica que cada servicio que se intenta desplegar **cumple un estándar de calidad definido**, y decide automáticamente si puede pasar, necesita revisión humana, o está bloqueado.

---

## El problema que resuelve

Sin un conformance gate, un equipo de desarrollo puede:

- Hacer push de una imagen Docker sin SBOM ni firma
- Desplegar un servicio sin healthcheck definido
- Entregar una API sin contrato OpenAPI exportable
- Romper la arquitectura hexagonal acordada sin que nadie lo detecte hasta producción

El gate automatiza esa verificación en cada PR, antes de que nada llegue a main.

---

## El diagrama completo — qué es cada caja

### Nivel 1: Generación de la "caja"

```
LM ANTHROPIC
└── Claude Code / Codex / Antigravity Experts
    Scaffolding + Skills + Agents + Sub-agents + Hooks
        └── genera → BACKEND "CAJA" (Rust/Axum)
```

**Qué hace:** Un agente IA (corriendo en el IDE del developer) construye el servicio backend siguiendo una especificación. El agente conoce las reglas de calidad de antemano (de `policy/conformity-policy.yaml`) y las aplica al generar el código.

**Lo que produce:** Un servicio Rust con arquitectura hexagonal, endpoints definidos con OpenAPI, manejo de errores tipado, sin panics en handlers.

**En el PoC:** Hay **dos** servicios Rust:

| Servicio       | Propósito                                | OCI compliant | Calidad BOX |
| -------------- | ---------------------------------------- | ------------- | ----------- |
| `rust-svc`     | Caja conforme — escenarios 1, 2, 3, 5    | ✅            | ✅          |
| `rust-svc-bad` | Caja intencionalmente mala — escenario 4 | ✅            | ❌          |

**Por qué dos servicios:** `rust-svc-bad` pasa todos los checks OCI (firma, SBOM, healthcheck, labels) pero viola las reglas de calidad BOX intencionalmente. Esto es indispensable: sin esta caja, OpenCode nunca se ejercita en los tests y el PoC no demuestra su valor.

Las violaciones en `rust-svc-bad` son deliberadas y documentadas:

- **BOX-001**: `/news` devuelve array crudo, `/health` usa `{success, data}` — shapes inconsistentes
- **BOX-002**: `.unwrap()` en handler — pánico potencial si `NEWS_COUNT` no es número
- **BOX-003**: lógica de negocio inline en el handler, sin separación `domain/`

---

### Nivel 2: El pipeline CI/CD

```
PUSH → CI/CD (GitHub Actions) en Rxcxrdx/poc-agent-devops
    ├── JOB 1: cargo test + clippy   ← compilación + linting + tests unitarios
    ├── JOB 2: SonarCloud            ← análisis estático (pasa en AMBOS servicios — demuestra su limitación)
    └── JOB 3: ★ CONFORMANCE GATE   ← último paso — uses: Rxcxrdx/conformance-agent@v1
```

**Por qué el gate va último:** Los pasos anteriores filtran los problemas mecánicos baratos de detectar. El gate LLM solo se ejecuta sobre cajas que ya pasaron esos filtros, minimizando costo de API y falsos positivos.

**El gate NO reemplaza SAST** — lo complementa. `cargo audit` detecta CVEs en dependencias. El gate detecta conformidad semántica y arquitectural de la caja: cosas que ninguna herramienta estática puede ver.

**Por qué no está SonarQube en el PoC — y por qué eso es intencional:**

Esta es una decisión de diseño deliberada, no una omisión. La propuesta de valor del agente se demuestra con este contraste en el escenario 4 (`rust-svc-bad`):

```
cargo clippy   →  ✅ pasa   (no hay antipatrones mecánicos)
cargo audit    →  ✅ pasa   (no hay CVEs en dependencias)
SonarQube      →  ✅ pasaría también (no hay vulnerabilidades de seguridad)
conftest Rego  →  ✅ pasa   (OCI labels, SBOM, firma, healthcheck: todo correcto)
               ─────────────────────────────────────────────────────────────
OpenCode OSS   →  ❌ falla  (BOX-001 + BOX-002 + BOX-003 detectados)
```

`rust-svc-bad` es código Rust válido, sin CVEs, sin code smells detectables por herramientas estáticas — pero viola contratos semánticos y arquitecturales que solo el razonamiento del agente puede ver. Agregar SonarQube al pipeline no cambiaría ese resultado: también pasaría. Lo que hace la historia del PoC más fuerte es precisamente que **todas** las herramientas deterministas dicen "está bien" y el agente dice "no".

La línea divisoria es:

- **Herramientas estáticas** (clippy, audit, SonarQube, Rego) → evalúan _estructura y reglas conocidas_
- **Agente OpenCode** → evalúa _semántica, arquitectura y comportamiento implícito_

---

### Nivel 3: El Conformance Gate (el PoC)

```
COMPLIANCE GATE
    │
    ├── Step A: conftest + Rego policies
    │     Reglas deterministas: firma, SBOM, healthcheck, labels, user, openapi
    │     Sin LLM. Rápido. Binario. Si falla → block inmediato.
    │
    ├── Step B: OpenCode OSS (ms client sdk)
    │     Agente que inspecciona código fuente y contrato API
    │     Evalúa: arquitectura hexagonal, homogeneidad de respuestas, resiliencia
    │     Usa: anthropic/claude-sonnet-4-6 (ágil, económico)
    │
    └── Step C: Claude Opus (Anthropic SDK directo)
          Solo se invoca si Step A pasó pero Step B encontró problemas "high"
          Reflexión profunda: ¿es un bloqueo real o falso positivo?
          Produce: adjusted_confidence + recommendation
```

**Por qué tres pasos y no uno:** Jerarquía de costo/velocidad/precisión.

- Step A: microsegundos, cero costo, cubre el 70% de los casos
- Step B: segundos, costo bajo (Sonnet), cubre el 25% restante
- Step C: segundos, costo medio (Opus), solo para el 5% ambiguo

---

### Nivel 4: La decisión y sus consecuencias

```
ConformanceDecision {
  decision: "pass" | "manual_review" | "block"
  confidence: 0.0 – 1.0
}
    │
    ├── pass (confidence ≥ 0.95)
    │     └── deploy automático a staging
    │
    ├── manual_review (0.80 ≤ confidence < 0.95)
    │     └── GitHub Environment "staging" pausa
    │         aprobador designado recibe notificación
    │         PR no puede mergear hasta aprobación o timeout
    │
    └── block (cualquier violación critical)
          └── check de GitHub falla
              merge a main bloqueado por política de branch protection
```

**El audit log** guarda cada decisión con evidencia completa: qué reglas se evaluaron, qué encontró cada step, con qué confianza, quién aprobó.

---

### Nivel 5: El ecosistema completo (más allá del PoC)

Las cajas naranjas del diagrama que **no están en el PoC** pero son el destino:

**COMPONENTE MS (Node + SDK Anthropic)**

- Un microservicio TypeScript que orquesta múltiples evaluaciones
- Usa Claude Opus directamente para workflows complejos de reflexión
- Expone una API HTTP para integrar con otros pipelines
- En el PoC: está fusionado dentro del motor de conformidad

**Agente Autónomo No Supervisado**

- Trigger de pipelines completos
- Experimentos de API, colas, eventos
- Sin intervención humana en el loop feliz
- En el PoC: no está incluido — es la evolución post-PoC

**OpenCode OSS**

- El runtime del agente (corriendo como servidor `opencode serve`)
- Soporta 75+ modelos via Models.dev
- Anti lock-in: cambiar de Claude a GPT-5 o Qwen sin tocar CI
- En el PoC: se usa exactamente así — como sidecar en CI

**Qwen Alibaba**

- Fallback de modelo alternativo
- Útil para reducir costos en checks de alta frecuencia
- En el PoC: no está incluido — roadmap post-PoC

**fe / IaC / GitOps**

- El frontend de gestión del sistema
- Infraestructura como código (Terraform, Pulumi)
- GitOps para sincronización de estado deseado vs real
- En el PoC: no están incluidos

---

## Lo que el PoC demuestra

Al finalizar el PoC, puedes mostrar esto con evidencia en el audit log:

| #     | Escenario        | Caja                  | Step que actúa        | Lo que ocurre                                                | Lo que demuestra                                      |
| ----- | ---------------- | --------------------- | --------------------- | ------------------------------------------------------------ | ----------------------------------------------------- |
| 1     | Verde            | `rust-svc`            | — ninguno falla       | Gate pasa, deploy a staging                                  | El sistema no bloquea trabajo válido                  |
| 2     | Rojo OCI         | `rust-svc` sin SBOM   | Step A (conftest)     | Gate bloquea, OpenCode **no se llama**                       | Critical se resuelve sin LLM — rápido y económico     |
| 3     | Ámbar OCI        | `rust-svc` sin labels | Step A (conftest)     | Manual review, humano debe aprobar                           | El sistema escala a humano en grises de metadatos     |
| **4** | **Calidad mala** | **`rust-svc-bad`**    | **Step B (OpenCode)** | **OpenCode encuentra BOX-001/002/003, conftest no vio nada** | **OpenCode agrega valor real sobre conftest**         |
| 5     | Regresión x5     | `rust-svc`            | — ninguno             | Mismo resultado en 5 corridas                                | El agente es estable, no hay drift entre evaluaciones |

**El escenario 4 es la prueba de fuego del PoC.** Si `evidence.deterministic_findings` está vacío y `evidence.llm_findings` tiene los BOX rules, el sistema demostró que el agente IA ve cosas que las reglas deterministas no pueden ver.

---

## Por qué importa tener dos cajas (`rust-svc` y `rust-svc-bad`)

Sin `rust-svc-bad`, los escenarios 1–3 y 5 solo validan conftest. Eso **no es un PoC agéntico** — es un PoC de OPA con CI, que ya existe en el mercado.

La diferencia entre conftest y OpenCode:

| ¿Detecta...?                             | conftest (Rego)                 | OpenCode (LLM) |
| ---------------------------------------- | ------------------------------- | -------------- |
| SBOM ausente                             | ✅ — regla binaria              | ✅             |
| Label faltante                           | ✅ — campo presente o no        | ✅             |
| Healthcheck definido                     | ✅ — campo en imagen            | ✅             |
| Shapes inconsistentes entre endpoints    | ❌ — requiere leer código       | ✅             |
| `.unwrap()` en handler (riesgo de panic) | ❌ — análisis semántico         | ✅             |
| Ausencia de separación domain/routes     | ❌ — razonamiento arquitectural | ✅             |

La línea divisoria es: **Rego evalúa estructura, OpenCode evalúa semántica y arquitectura.**

---

## La política como contrato compartido

`policy/conformity-policy.yaml` es el documento central. Es lo que:

1. **El agente del IDE lee** para saber qué estándares debe cumplir al generar código
2. **Conftest evalúa** para los checks deterministas
3. **El agente OpenCode lee** como contexto para los checks cualitativos
4. **Los humanos revisan** para auditar o cambiar las reglas

Si el agente que genera código conoce exactamente las mismas reglas que el agente que verifica, el ciclo se cierra: _lo que se generó bien, pasa bien_.

---

## Glosario

| Término              | Definición en este sistema                                                               |
| -------------------- | ---------------------------------------------------------------------------------------- |
| **Caja**             | Un servicio backend empaquetado como imagen OCI, listo para desplegar                    |
| **Caja conforme**    | Caja que cumple todas las reglas OCI y BOX — pasa el gate sin intervención               |
| **Caja no conforme** | Caja con al menos una violación — en el PoC: `rust-svc-bad` (viola BOX intencionalmente) |
| **Conformidad**      | Cumplimiento verificable de un conjunto de reglas definidas en política                  |
| **Gate**             | Punto de control en CI/CD que puede pasar, pausar o bloquear el flujo                    |
| **Conftest**         | CLI de OPA para evaluar JSON/YAML contra políticas Rego — evalúa _estructura_            |
| **OpenCode**         | Runtime de agente IA — evalúa _semántica y arquitectura_ via LLM                         |
| **Opus reflection**  | Paso de razonamiento profundo solo para casos ambiguos (cost-aware)                      |
| **Audit log**        | Registro inmutable de cada decisión del gate con evidencia completa                      |
| **manual_review**    | Estado intermedio donde un humano aprueba o rechaza antes de continuar                   |
| **Cortocircuito**    | Si Step A encuentra critical → block inmediato, OpenCode no se llama                     |

---

## Evolución post-PoC (roadmap)

```
PoC (ahora)
├── 2 cajas ejemplo (rust-svc conforme + rust-svc-bad intencionalmente mala)
├── Política con 10 reglas (7 OCI + 3 BOX)
├── 3 steps (conftest + OpenCode + Opus)
├── 5 escenarios de prueba con assertions verificables
└── CI en un repositorio con GitHub Actions

v1.0 (siguiente)
├── Múltiples cajas (cualquier servicio del equipo)
├── Política extendida (DORA metrics, SLO, contrato de dependencias)
├── COMPONENTE MS como servicio independiente
└── Dashboard de conformidad con métricas históricas

v2.0 (futuro)
├── Agente autónomo no supervisado (auto-remediation en rama separada)
├── Multi-model (Qwen fallback para reducir costos)
├── Políticas OPA más granulares (por tipo de servicio, por team)
└── Integración con GitOps (estado deseado vs estado real en producción)
```

---

## Archivos del proyecto y su propósito

| Archivo                             | Propósito                                                                              |
| ----------------------------------- | -------------------------------------------------------------------------------------- |
| `policy/conformity-policy.yaml`     | Fuente de verdad de las reglas — legible por humanos y agentes                         |
| `policy/rego/*.rego`                | Implementación técnica de las reglas deterministas (OPA/Conftest)                      |
| `services/rust-svc/`                | Caja **conforme** — escenarios 1, 2, 3, 5                                              |
| `services/rust-svc-bad/`            | Caja **intencionalmente mala** — escenario 4, demuestra OpenCode                       |
| `conformance/src/engine.ts`         | Orquestador de los 3 steps del gate                                                    |
| `conformance/src/opencode-agent.ts` | Integración con OpenCode SDK — el paso LLM cualitativo                                 |
| `conformance/src/cli.ts`            | Punto de entrada para CI (devuelve JSON a stdout)                                      |
| `caja-manifests/`                   | 5 manifests de prueba (compliant, missing-sbom, missing-labels, bad-quality, fixtures) |
| `audit-log/`                        | Evidencia de cada decisión (generado en runtime, gitignored)                           |
| `.github/workflows/`                | El pipeline CI que integra todo                                                        |
