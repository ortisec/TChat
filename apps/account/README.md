# apps/account — identidad

Proyecto Astro + React para **`app.tchat.site`**.

## Páginas

| Ruta        | Contenido                                              |
| ----------- | ------------------------------------------------------ |
| `/`         | Registro, inicio de sesión y perfil (misma isla React) |
| `/cli-auth` | Autorización del dispositivo que pide el terminal      |

## Autenticación y API

- Firebase Auth con **email/password** y **Google**.
- Tras iniciar sesión, la web llama a `POST /api/v1/auth/sync` con el ID token de Firebase y
  muestra el `@username` que asigna la API. Si la API no está levantada, el perfil sigue visible
  con la etiqueta «sin conexión con la API».
- `/cli-auth?code=XXXX-XXXX` pide sesión (o la reutiliza), muestra el código y lo aprueba contra
  `POST /api/v1/auth/device/approve`. El terminal recoge los tokens al instante.
- Los errores de Firebase se traducen a mensajes en español en `src/lib/auth-errors.ts`.
- Sin configuración de Firebase la app muestra un aviso con las variables que faltan.

## Configuración

Copia `apps/account/.env.example` a `apps/account/.env` y completa la configuración web de Firebase
(Console → Project settings → General → Your apps → Web app):

```env
PUBLIC_FIREBASE_API_KEY=...
PUBLIC_FIREBASE_APP_ID=...
PUBLIC_API_URL=http://localhost:8000
```

Los demás valores ya están puestos: proyecto `tchat-c20ce`, dominio `tchat-c20ce.firebaseapp.com`
y sender id `256761907628`.

## Desarrollo

```powershell
just api                          # backend en :8000
pnpm --filter @tchat/account dev  # web en :4322
```

**Estado:** Paso 4 completado. El terminal real llega en el Paso 5.
