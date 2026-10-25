# apps/cli — cliente de terminal

Workspace de Rust para el cliente de TChat.

```
crates/
├─ tchat-core/   # HTTP, WebSocket, auth, modelos, protocolo
├─ tchat-tui/    # Ratatui: pantallas, widgets, eventos
└─ tchat-cli/    # binario: args, config, keyring, bootstrap
```

## Comandos

```powershell
just cli-login   # device flow: abre el navegador y guarda la sesión
just cli         # abre la TUI
just check-cli   # fmt + clippy -D warnings + tests
```

O directamente:

```powershell
cargo run -p tchat-cli -- login
cargo run -p tchat-cli -- status
cargo run -p tchat-cli -- open bruno   # abre/crea el chat directo con @bruno
cargo run -p tchat-cli -- logout
cargo run -p tchat-cli
```

`open` requiere que ya seáis amigos; la amistad se solicita desde la API o la web.

## Configuración

| Variable            | Por defecto             | Uso                |
| ------------------- | ----------------------- | ------------------ |
| `TCHAT_API_URL`     | `http://localhost:8000` | API REST           |
| `TCHAT_WS_URL`      | derivada de la API      | WebSocket (`…/ws`) |
| `TCHAT_ACCOUNT_URL` | `http://localhost:4322` | Web de cuentas     |

También se pueden pasar como `--api-url`, `--ws-url` y `--account-url`.

## Sesión y tokens

- `tchat login` inicia el device flow, muestra el código y abre `app.tchat.ortisec.site/cli-auth`.
- El refresh token se guarda en el **llavero del sistema**:
  - Windows: Credential Manager (`windows-native`).
  - macOS: Keychain (`apple-native`).
  - Linux: keyutils (`linux-native`). El token vive en el keyring del kernel de la sesión;
  puede ser necesario volver a ejecutar `tchat login` tras reiniciar.
- El access token dura 15 minutos y el cliente lo renueva solo; al agotarse el refresh hay que
  volver a ejecutar `tchat login`.
- `tchat logout` revoca el token en la API y lo borra del llavero.

## TUI

| Tecla        | Acción                 |
| ------------ | ---------------------- |
| Enter        | Envía el mensaje       |
| ↑ / ↓        | Cambia de conversación |
| PgUp / PgDn  | Desplaza el historial  |
| Esc / Ctrl+C | Sale                   |

Recibe mensajes, presencia y acuses por WebSocket. Si la conexión cae, reintenta con backoff y
renueva el token automáticamente.

## Tests

```powershell
cargo test --workspace
```

Cubren el sobre del protocolo, la decodificación de eventos, la configuración, los modelos y las
transiciones de estado de la TUI (17 tests). No requieren red ni llavero.

## Release

Los binarios se publican con cargo-dist al empujar un tag `tchat-cli-v*`:
Windows (zip + msi), macOS (Intel y Apple Silicon), Linux (glibc y musl), instaladores
`install.sh`/`install.ps1` y checksums. Ver `docs/runbooks/release-cli.md`.

**Estado:** Paso 7 completado.
