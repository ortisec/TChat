# apps/account — identidad

Proyecto Astro + React para **`app.tchat.site`**.

## Páginas

| Ruta        | Contenido                                              |
| ----------- | ------------------------------------------------------ |
| `/`         | Registro, inicio de sesión y perfil (misma isla React) |
| `/cli-auth` | Autorización del dispositivo que pide el terminal      |

## Autenticación

- Firebase Auth con **email/password** y **Google**.
- El formulario de registro crea el usuario en Firebase y guarda el nombre visible.
- Los errores de Firebase se traducen a mensajes en español en `src/lib/auth-errors.ts`.
- Sin configuración de Firebase la app muestra un aviso con las variables que faltan, en lugar de
  romperse.

## Configuración

Copia `apps/account/.env.example` a `apps/account/.env` y completa la configuración web de Firebase
(Console → Project settings → General → Your apps → Web app):

```env
PUBLIC_FIREBASE_API_KEY=...
PUBLIC_FIREBASE_APP_ID=...
```

Los demás valores ya están puestos: proyecto `tchat-c20ce`, dominio `tchat-c20ce.firebaseapp.com`
y sender id `256761907628`.

## Desarrollo

```powershell
pnpm --filter @tchat/account dev   # http://localhost:4322
```

**Estado:** Paso 3 completado. La sincronización del perfil con la API y la aprobación real del
device flow llegan en el Paso 4.
