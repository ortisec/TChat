# Release del cliente

## Formato del tag

El workspace de Cargo vive en `apps/cli` y el paquete distribuible es `tchat-cli`. cargo-dist usa
el formato `<paquete>-v<versión>`:

```
tchat-cli-v0.1.0
```

Ese prefijo (`tag-namespace = "tchat-cli"`) hace que solo ese workflow reaccione a los tags del
cliente, sin interferir con los despliegues de la API.

## Antes de publicar

1. Sube la versión en `apps/cli/Cargo.toml` (`[workspace.package] version`).
2. Ejecuta `just check` (incluye fmt, clippy, tests y el resto del monorepo).
3. Opcional: añade notas en `apps/cli/CHANGELOG.md`; cargo-dist las usa si existen.

## Publicar

```bash
git tag tchat-cli-v0.1.0
git push origin tchat-cli-v0.1.0
```

El workflow **Release** (`.github/workflows/tchat-cli-release.yml`):

1. `plan` calcula artefactos y crea la release.
2. `build-local-artifacts` compila en 5 targets:
   - Windows x86_64 (`windows-2022`, incluye WiX para el MSI)
   - macOS x86_64 e Apple Silicon
   - Linux x86_64 glibc y musl
3. `build-global-artifacts` genera los instaladores (`tchat-cli-installer.sh`, `.ps1`) y checksums.
4. `host` sube todo a GitHub Releases.
5. `custom-latest-json` ejecuta `infra/scripts/latest-json.sh` y adjunta `latest.json`.

Duración típica: 10–15 minutos.

## Verificar

```bash
gh release view tchat-cli-v0.1.0

curl -fsS https://cli.tchat.ortisec.site/latest.json | jq .
curl -fsS https://cli.tchat.ortisec.site/install.sh | sh
tchat --version
```

En Windows:

```powershell
irm https://cli.tchat.ortisec.site/install.ps1 | iex
tchat --version
```

Comprueba también que los enlaces de `https://cli.tchat.ortisec.site` muestran la versión nueva y que el
CLI sigue autenticando.

## Rollback

No se puede "despublicar" un binario ya descargado. Para corregir:

1. Publica una versión superior con el arreglo (recomendado).
2. Si la release está rota, márcala como _draft_ o elimínala, y vuelve a lanzar el tag anterior:

   ```bash
   gh release delete tchat-cli-v0.1.0 --yes
   git tag -d tchat-cli-v0.1.0
   git push origin :tchat-cli-v0.1.0
   ```

## Mantenimiento del workflow

`tchat-cli-release.yml` lo genera cargo-dist **y se parchea a mano** porque el workspace no está
en la raíz del repositorio. Por eso `allow-dirty = ["ci"]` está activo.

Si actualizas cargo-dist:

1. Quita temporalmente `allow-dirty` de `apps/cli/Cargo.toml`.
2. Regenera:
   ```bash
   cd apps/cli
   dist generate --mode ci
   dist generate --mode msi
   ```
3. Vuelve a aplicar el parche:
   - `env: CARGO_TARGET_DIR: ${{ github.workspace }}/target` a nivel de workflow.
   - Envolver cada invocación de `dist` con `(cd apps/cli && dist …)` manteniendo la redirección
     del manifiesto en la raíz.
4. Restaura `allow-dirty = ["ci"]` y valida con `dist plan --tag tchat-cli-vX.Y.Z`.

## Comprobaciones de seguridad

- Los checksums SHA-256 se publican por artefacto y en `latest.json`.
- Los instaladores se generan a partir de esos artefactos; revisa el diff del workflow si cambia
  cargo-dist de versión.
- Las releases son inmutables por convención: no reemplaces artefactos de un tag ya publicado.
