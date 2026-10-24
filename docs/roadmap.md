# Roadmap de construcción

Siete pasos. Cada paso deja el repositorio en un estado coherente, con CI en verde y algo
demostrable. Al final de cada bloque se indica **tu parte**: lo que solo puedes hacer tú
(paneles, secretos, cuentas).

Estado actual: **Paso 6 de 7 completado**.

---

## Paso 1 — Bootstrap del monorepo ✅

**Entregables**

- Raíz delgada: `package.json`, `pnpm-workspace.yaml`, `justfile`.
- Higiene de git: `.gitignore`, `.gitattributes` (LF en repo, CRLF en `.ps1`), `.editorconfig`.
- Estructura completa de carpetas con un `README.md` por superficie.
- `docs/architecture.md`, este roadmap y los primeros ADR.
- CI base con detección de cambios: los jobs de cada stack se activan solos cuando aparece su código.

**Tu parte**

1. Abre una terminal nueva para que `just` entre en el `PATH` y comprueba `just --list`.
2. Guarda el diagrama en `docs/assets/arquitectura.png`.
3. En GitHub (`ortisec/TChat`): descripción, topics (`chat`, `terminal`, `rust`, `fastapi`, `astro`), desactivar Wiki/Projects si no los usas.
4. Proteger `main`: exigir PR y el check `CI` antes de fusionar.
5. Decidir licencia (si es MIT, añadir `LICENSE`; si no, dejar `UNLICENSED`).
6. Confirmar el primer commit y push.

**Criterio de hecho:** `just check` pasa, `git status` limpio tras el commit, CI en verde
(solo corre el job de detección porque aún no hay código de ningún stack).

---

## Paso 2 — Backend base y contrato ✅

**Entregables**

- `services/api` con uv, FastAPI, Pydantic Settings y estructura por capas.
- Modelos SQLAlchemy y primera migración Alembic (`users`).
- Endpoint `GET /health` y `GET /api/v1/meta`.
- Exportador de OpenAPI → `packages/contracts/openapi.json`.
- `packages/api-client` generado con `openapi-typescript` + `openapi-fetch`.
- CI: ruff, mypy, pytest con PostgreSQL de servicio, y job `contract` que detecta desfases.

**Tu parte**

1. ✅ Proyecto de Firebase creado: `tchat-c20ce`.
2. ✅ `FIREBASE_PROJECT_ID` recibido y configurado como valor por defecto.
3. Pendiente para el Paso 6: descargar la service account (Project settings → Service accounts
   → Generate key) y copiarla al VPS. **No la subas al repo.**

**Criterio de hecho:** `uv run pytest` en verde, `just api` levanta en `:8000/docs`, y
`just gen-client` deja el cliente TS compilando.

---

## Paso 3 — Las tres apps web ✅

**Entregables**

- `apps/www`, `apps/account`, `apps/download` como proyectos Astro independientes.
- Tailwind CSS con una única escala de grises (sin colores de acento), tokens en `packages/ui`.
- `packages/config` con tsconfig, eslint y prettier compartidos.
- `apps/account`: registro, login, logout, perfil y página `cli-auth` para el device flow.
- `apps/download`: detección de SO, enlaces a releases y scripts de instalación.
- `apps/www`: landing que explica el producto.

**Tu parte**

1. ✅ Email/Password y Google habilitados en Firebase.
2. Crear (si no existe) la **Web App** en Firebase Console → Project settings → General → Your
   apps. Copiar `apiKey` y `appId` a `apps/account/.env` y a las variables de Vercel.
3. Crear tres proyectos en Vercel apuntando al mismo repo, cada uno con su _Root Directory_
   (`apps/www`, `apps/account`, `apps/download`).
4. Asignar dominios: `tchat.site`, `app.tchat.site`, `cli.tchat.site`.
5. Añadir esos dominios en Firebase → Authentication → Settings → Authorized domains.
6. Pegar las variables `PUBLIC_*` en cada proyecto de Vercel.

**Criterio de hecho:** las tres URLs cargan en grises, el registro crea usuario en Firebase y
`/cli-auth` pide la cuenta. Verificado en local con build de producción; el registro real queda
pendiente de las claves web de Firebase (`apiKey` y `appId`).

---

## Paso 4 — API funcional ✅

**Entregables**

- Verificación de ID tokens con Firebase Admin.
- `/auth/sync`, `/auth/device/*`, `/auth/token`, `/auth/logout`.
- Usuarios, amistades (solicitar/aceptar/rechazar), conversaciones y mensajes.
- WebSocket con autenticación, entrega de mensajes, acuse y presencia.
- Migraciones para todas las tablas y tests de los flujos críticos.

**Tu parte**

1. ✅ No hace falta nada para probar: los tests crean su propia base `tchat_test`.
2. Para usar la web contra la API en local: `just api` y haber pegado `apiKey`/`appId` de
   Firebase en `apps/account/.env` (Paso 3).
3. Opcional: fijar reglas de negocio (límite de caracteres, grupos o solo 1:1).

**Criterio de hecho:** dos clientes de prueba intercambian mensajes por WebSocket y las
amistades se gestionan por REST. Verificado con 29 tests de integración contra PostgreSQL,
más la capa WebSocket cubierta con tests de conexión y protocolo.

---

## Paso 5 — Cliente Rust ✅

**Entregables**

- `tchat-core`: cliente HTTP, WebSocket, refresco de tokens, protocolo Serde.
- `tchat-tui`: Ratatui con login, lista de conversaciones, mensajes y caja de escritura.
- `tchat-cli`: `tchat login`, `tchat logout`, `tchat open <usuario>`, `tchat` (TUI) y
  `tchat --version`.
- Almacenamiento del refresh token en el keyring del SO.
- Tests unitarios del protocolo y del parseo de configuración.

**Tu parte**

1. Prueba la TUI en Windows (`just cli-login` y luego `just cli`) y cuéntame los fallos visuales:
   altura de paneles, acentos, teclas, comportamiento del scroll.
2. Si tienes una Mac o Linux a mano, valida ahí también (en Linux el llavero usa keyutils).
3. Necesitas la API levantada (`just api`) y una conversación creada para ver mensajes.

**Criterio de hecho:** `tchat login` autentica por navegador y permite chatear contra la API local.
Verificado el flujo completo en Windows (device flow, keyring, `status`, `logout`) y la TUI con
tests de estado y protocolo; falta la validación visual interactiva.

---

## Paso 6 — Infraestructura y despliegue ✅

**Entregables**

- `infra/compose/docker-compose.yml` (dev) y `.prod.yml` (VPS).
- `Caddyfile` con TLS automático para `api.` y `ws.`.
- `Dockerfile` multi-stage para la API.
- Workflow `deploy-api.yml`: GHCR → SSH → migraciones → `up -d`.
- Backups de PostgreSQL y runbook de restauración.

**Tu parte**

1. Contratar el VPS (Ubuntu 24.04 con Docker) y darme IP y usuario SSH.
2. Apuntar los DNS `api.tchat.site` y `ws.tchat.site` al VPS y abrir solo 22/80/443.
3. Crear el Environment `production` en GitHub con `VPS_HOST`, `VPS_USER`, `VPS_SSH_KEY` y,
   si no usas el puerto 22, `VPS_PORT`.
4. Copiar `infra/compose/.env.example` a `/opt/tchat/infra/compose/.env` en el VPS y rellenar
   `POSTGRES_PASSWORD`, `AUTH_JWT_SECRET` y `ACME_EMAIL`.
5. Opcional: la service account de Firebase solo hace falta si añadimos operaciones
   administrativas; la verificación de tokens no la necesita.

**Criterio de hecho:** `https://api.tchat.site/health` responde y el CLI funciona contra producción.
Verificado en local: imagen Docker, pila completa contra PostgreSQL, `alembic check` dentro del
contenedor, compose de producción y Caddyfile validados. Falta ejecutarlo en tu VPS.

---

## Paso 7 — Release del CLI y CI/CD completo

**Entregables**

- `cargo-dist` configurado: `.msi`, `.tar.gz`, checksums e instaladores.
- Workflow de release por tag `cli-v*` para Windows, Linux (glibc y musl) y macOS (x86_64 y arm64).
- `latest.json` y scripts `install.sh` / `install.ps1` servidos desde `cli.tchat.site`.
- CI/CD completo documentado y `docs/runbooks/`.

**Tu parte**

1. Probar la instalación en una máquina limpia de Windows (`install.ps1`).
2. Revisar que los binarios de macOS/Linux arrancan (o pasármelo para probar en CI).
3. Definir la versión inicial (`v0.1.0`).

**Criterio de hecho:** un usuario nuevo instala con un comando, ejecuta `tchat login` y chatea.
