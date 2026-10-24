# 0001 — Monorepo único para los tres runtimes

- **Estado:** aceptado
- **Fecha:** 2026-10-24

## Contexto

TChat necesita un frontend web, un backend y un cliente de terminal. Los tres evolucionan juntos:
cada cambio de contrato afecta a la web y al CLI. Además, el proyecto lo mantiene una sola persona,
que trabaja en Windows.

## Decisión

Un único repositorio con separación por capa de despliegue:

- `apps/` — lo que corre en la máquina del usuario (`www`, `account`, `download`, `cli`).
- `services/` — lo que corre en el servidor (`api`).
- `packages/` — código compartido solo entre superficies JS.
- `infra/` — despliegue y operación.
- `docs/` — arquitectura y decisiones.

La raíz contiene únicamente el workspace JS y el `justfile`. Cada stack conserva su manifiesto
y su lockfile dentro de su carpeta (`apps/*/package.json`, `apps/cli/Cargo.toml`,
`services/api/pyproject.toml`).

## Consecuencias

- Un PR puede cambiar API + web + CLI y CI valida el conjunto.
- No hay publicación de paquetes internos ni versionado cruzado.
- El clonado es algo más pesado y CI necesita filtros por rutas para no ejecutar todo siempre.
- Vercel requiere configurar *Root Directory* por proyecto y activar el soporte de monorepo.

## Alternativas descartadas

- **Repos separados:** obliga a coordinar versiones de contrato a mano y multiplica la configuración.
- **Un solo paquete con todo dentro:** mezcla entornos y hace imposible desplegar por separado.
