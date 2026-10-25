# 0002 — Tres proyectos web independientes

- **Estado:** aceptado
- **Fecha:** 2026-10-24

## Contexto

La arquitectura define tres subdominios con propósitos distintos: `tchat.ortisec.site` (landing),
`app.tchat.ortisec.site` (registro, login y perfil) y `cli.tchat.ortisec.site` (descargas). Se evaluó un único
proyecto Astro con reescrituras por host frente a tres proyectos separados.

## Decisión

Tres proyectos Astro en `apps/www`, `apps/account` y `apps/download`, cada uno con su
`package.json`, su configuración y su proyecto en Vercel.

Comparten `packages/ui`, `packages/config` y `packages/api-client` mediante el workspace de pnpm.

## Consecuencias

- Cada dominio se despliega y revierte por separado; un fallo en descargas no tumba el login.
- Los bundles son mínimos: la landing no carga el SDK de Firebase.
- Cuesta mantener tres `astro.config.mjs` y tres `package.json`; se mitiga con `packages/config`.
- Los cambios de diseño compartido se hacen una vez en `packages/ui`.

## Alternativas descartadas

- **Un proyecto con rewrites por host:** menos archivos, pero acopla despliegues y arrastra el
  SDK de autenticación a páginas que no lo necesitan.
- **Un proyecto por página con dominio propio:** fragmentación excesiva para el tamaño actual.
