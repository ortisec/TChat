## Qué cambia

<!-- Resume el cambio en una o dos frases. -->

## Paso del roadmap

<!-- Paso 1..7 según docs/roadmap.md -->

## Tipo de cambio

- [ ] Web (`apps/www`, `apps/account`, `apps/download`)
- [ ] API (`services/api`)
- [ ] CLI (`apps/cli`)
- [ ] Contrato (`packages/contracts`, `packages/api-client`)
- [ ] Infraestructura (`infra/`, `.github/`)
- [ ] Documentación

## Checklist

- [ ] `just check` pasa en local
- [ ] Si cambié la API, regeneré `openapi.json` y `packages/api-client`
- [ ] Si cambié el protocolo WS, actualicé `packages/contracts/ws-events.md`
- [ ] No hay secretos ni `.env` en el diff
- [ ] Actualicé la documentación afectada
