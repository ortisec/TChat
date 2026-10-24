# infra — despliegue y operación

```
compose/    # docker-compose.yml (dev) y docker-compose.prod.yml (VPS)
caddy/      # Caddyfile: TLS y proxy para api. y ws.
scripts/    # install.sh, install.ps1, backup.sh, deploy.sh
```

Reglas:

- Las imágenes de la API se publican en GHCR y el VPS solo hace `pull`.
- Las migraciones se ejecutan antes de levantar la nueva versión.
- PostgreSQL nunca expone su puerto fuera de la red de Docker.
- Los secretos se inyectan por variables de entorno, nunca en la imagen.

**Estado:** pendiente (Paso 6).
