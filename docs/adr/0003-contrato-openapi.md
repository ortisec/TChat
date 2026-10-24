# 0003 — Contrato OpenAPI como fuente de verdad

- **Estado:** aceptado
- **Fecha:** 2026-10-24

## Contexto

La web y el CLI consumen la misma API. Mantener tipos a mano provoca desfases silenciosos:
el backend renombra un campo y el cliente falla en producción.

## Decisión

1. FastAPI es la única fuente del contrato REST.
2. Un comando exporta `packages/contracts/openapi.json` y se versiona en git.
3. `packages/api-client` se genera con `openapi-typescript` y se envuelve con `openapi-fetch`.
4. CI vuelve a exportar el contrato y falla si difiere del versionado.

El protocolo WebSocket se documenta en `packages/contracts/ws-events.md` y se implementa con
Pydantic en el backend y Serde en Rust; los nombres de eventos son el contrato.

## Consecuencias

- Cambiar la API obliga a regenerar y commitear el contrato en el mismo PR.
- La web obtiene autocompletado real sin escribir tipos a mano.
- Se añade un paso de generación antes de compilar la web.
- El CLI queda fuera de la generación: valida con tests de contrato propios.

## Alternativas descartadas

- **tRPC / GraphQL:** no encajan con un cliente Rust ni con REST sobre WebSocket.
- **Tipos escritos a mano:** el problema que esta decisión evita.
