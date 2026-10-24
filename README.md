# TChat

Chat de terminal. Autenticación web, identidad propia y mensajería en tiempo real desde la terminal.

> **Estado: Paso 3 de 7 — tres apps web.**
> Hoja de ruta completa en [docs/roadmap.md](docs/roadmap.md).

## La arquitectura en una frase

El usuario se registra en la web (Firebase Auth), descarga el cliente Rust y chatea desde la terminal
contra FastAPI + PostgreSQL en el VPS, con WebSocket para mensajería y presencia.

## Superficies del producto

| Superficie                | Carpeta                          | Dominio                            | Despliegue       |
| ------------------------- | -------------------------------- | ---------------------------------- | ---------------- |
| Landing pública           | `apps/www`                       | `tchat.site`                       | Vercel           |
| Registro / login / perfil | `apps/account`                   | `app.tchat.site`                   | Vercel           |
| Descargas e instaladores  | `apps/download`                  | `cli.tchat.site`                   | Vercel           |
| Cliente de terminal       | `apps/cli`                       | —                                  | GitHub Releases  |
| API REST + WebSocket      | `services/api`                   | `api.tchat.site` / `ws.tchat.site` | VPS (Docker)     |
| Base de datos             | `services/api` + `infra/compose` | —                                  | VPS (Docker)     |
| Autenticación             | Firebase Auth                    | —                                  | Google (externo) |

## Estructura del repositorio

```
tchat/
├─ apps/                 # todo lo que corre en la máquina del usuario
│  ├─ www/               # Astro + React · landing
│  ├─ account/           # Astro + React · auth y perfil
│  ├─ download/          # Astro + React · descargas
│  └─ cli/               # Rust · tchat-core + tchat-tui + tchat-cli
├─ services/
│  └─ api/               # FastAPI + SQLAlchemy + PostgreSQL + WebSocket
├─ packages/             # compartido entre las superficies JS
│  ├─ ui/                # componentes React y tokens de diseño
│  ├─ api-client/        # cliente TS generado desde OpenAPI
│  ├─ config/            # tsconfig / eslint / prettier base
│  └─ contracts/         # openapi.json + protocolo WebSocket
├─ infra/
│  ├─ compose/           # docker-compose dev y prod
│  ├─ caddy/             # proxy inverso + TLS
│  └─ scripts/           # instaladores y utilidades de operación
├─ docs/
│  ├─ architecture.md    # arquitectura detallada
│  ├─ roadmap.md         # los 7 pasos de construcción
│  └─ adr/               # decisiones de arquitectura registradas
└─ .github/              # CI/CD, plantillas y dependabot
```

## Requisitos locales

| Herramienta    | Versión | Instalación                           |
| -------------- | ------- | ------------------------------------- |
| Node.js        | 24 LTS  | `winget install OpenJS.NodeJS.LTS`    |
| pnpm           | 12      | `npm install -g pnpm`                 |
| uv             | 0.9+    | `winget install astral-sh.uv`         |
| Rust           | stable  | `winget install Rustlang.Rustup`      |
| Docker Desktop | 29+     | `winget install Docker.DockerDesktop` |
| just           | 1.59+   | `winget install Casey.Just`           |

Tras instalar `just`, abre una terminal nueva para que el `PATH` se actualice.

## Primeros comandos

```powershell
just              # lista las recetas disponibles
just install      # instala dependencias JS y Python
just up           # PostgreSQL local en :5432
just migrate      # aplica migraciones
just api          # API en http://localhost:8000/docs
just dev          # las tres apps web en :4321, :4322 y :4323
just test         # tests de la API
just gen-client   # exporta OpenAPI y regenera el cliente TS
just check        # formato, lint y tipos de todo el monorepo
```

## Documentación

- [Arquitectura](docs/architecture.md)
- [Roadmap de 7 pasos](docs/roadmap.md)
- [Decisiones (ADR)](docs/adr/)

## Licencia

Pendiente de definir.
