# Backups y restauración

## Qué se guarda

Solo la base de datos. Los binarios del cliente viven en GitHub Releases y el frontend en Vercel;
ninguno necesita backup.

Formato: `pg_dump` custom comprimido con gzip. Los archivos se llaman
`tchat-AAAAAMMDDTHHMMSSZ.dump.gz`.

## Copia automática

En el VPS, añade una entrada de cron para el usuario que ejecuta Docker:

```cron
0 3 * * * /opt/tchat/infra/scripts/backup.sh >> /var/log/tchat-backup.log 2>&1
```

- Destino por defecto: `/opt/tchat/backups`.
- Retención: 14 días (`RETENTION_DAYS`).
- Variables útiles: `STACK_DIR`, `BACKUP_DIR`, `RETENTION_DAYS`.

Comprueba el resultado:

```bash
tail -n 20 /var/log/tchat-backup.log
ls -lh /opt/tchat/backups
```

## Copia manual

```bash
cd /opt/tchat
bash infra/scripts/backup.sh
```

## Copia fuera del VPS

Un backup que solo vive en el mismo disco no es un backup. Sincroniza el directorio a otro sitio:

```bash
# desde tu máquina
scp -r usuario@vps:/opt/tchat/backups ./tchat-backups
```

O configura `rclone`/restic con almacenamiento de objetos. Recomendado al menos semanal.

## Restauración

> ⚠️ La restauración reemplaza la base de datos completa. Haz una copia del estado actual antes.

1. Detén la API y Caddy para que nadie escriba durante la restauración:

   ```bash
   cd /opt/tchat
   docker compose -f infra/compose/docker-compose.prod.yml stop api caddy
   ```

2. Restaura (el script pide confirmación):

   ```bash
   bash infra/scripts/restore.sh /opt/tchat/backups/tchat-20261024T030000Z.dump.gz
   ```

   El script usa `pg_restore --clean --if-exists --no-owner`.

3. Vuelve a levantar:

   ```bash
   docker compose -f infra/compose/docker-compose.prod.yml up -d
   docker compose -f infra/compose/docker-compose.prod.yml exec api alembic current
   ```

4. Verifica:

   ```bash
   curl -fsS https://api.tchat.ortisec.site/health/ready
   ```

   Y revisa que existan usuarios y mensajes recientes:

   ```bash
   docker compose -f infra/compose/docker-compose.prod.yml exec postgres \
     psql -U tchat -d tchat -c "select count(*) from users; select count(*) from messages;"
   ```

## Prueba de restauración

Haz una prueba trimestral en una base desechable:

1. Crea una base `tchat_restore_test`.
2. Restaura el dump ahí con `pg_restore`.
3. Comprueba que las migraciones coinciden (`alembic check` con un `DATABASE_URL` apuntando a esa
   base).
4. Borra la base de prueba.

Una restauración que nunca se ha probado no cuenta como backup.

## Notas

- Las migraciones no se revierten con el backup, pero `pg_restore --clean` deja el esquema tal y
  como estaba en el momento del dump, incluida `alembic_version`.
- Si restauras un dump anterior a una migración, ejecuta `alembic upgrade head` después.
- El contenedor `postgres` no expone puertos: usa `docker compose exec` para cualquier operación.
