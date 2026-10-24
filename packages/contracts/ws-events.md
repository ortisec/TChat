# Protocolo WebSocket

Implementado en `services/api/src/tchat_api/realtime/`.

- URL: `wss://ws.tchat.site/ws`
- Autenticación: cabecera `Authorization: Bearer <token>` en el handshake. Acepta el access token
  del CLI y los ID tokens de Firebase.
- Formato: JSON con un sobre común.

## Sobre

```json
{
  "type": "message.send",
  "id": "0192f0c4-8f3a-7c2e-9c1b-1f4f2a7d3e55",
  "ts": "2026-10-24T18:00:00Z",
  "payload": {}
}
```

| Campo     | Tipo     | Descripción                            |
| --------- | -------- | -------------------------------------- |
| `type`    | string   | Nombre del evento                      |
| `id`      | uuid     | Identificador del evento (para acuses) |
| `ts`      | RFC 3339 | Marca de tiempo del emisor             |
| `payload` | object   | Cuerpo específico del evento           |

## Cliente → servidor

| `type`         | payload                                 | Descripción                            |
| -------------- | --------------------------------------- | -------------------------------------- |
| `ping`         | `{}`                                    | Keepalive; el servidor responde `pong` |
| `message.send` | `conversation_id`, `body`, `client_id?` | Envía un mensaje                       |
| `message.read` | `conversation_id`, `message_id`         | Marca como leído hasta ese mensaje     |
| `typing.start` | `conversation_id`                       | El usuario está escribiendo            |
| `typing.stop`  | `conversation_id`                       | Dejó de escribir                       |

`body` se recorta y admite entre 1 y 4000 caracteres. El usuario debe pertenecer a la conversación.

## Servidor → cliente

| `type`            | payload                                               | Descripción                |
| ----------------- | ----------------------------------------------------- | -------------------------- |
| `pong`            | `{}`                                                  | Respuesta a `ping`         |
| `message.new`     | `message` (mismo esquema que `MessagePublic` de REST) | Mensaje entrante           |
| `message.ack`     | `client_id`, `message_id`, `created_at`               | Confirmación de envío      |
| `presence.update` | `user_id`, `status` (`online` / `offline`)            | Presencia de un usuario    |
| `typing.update`   | `conversation_id`, `user_id`, `active`                | Actividad de escritura     |
| `error`           | `code`, `message`, `ref`                              | Error asociado a un evento |

## Errores

| `code`                   | Significado                                             |
| ------------------------ | ------------------------------------------------------- |
| `invalid_token`          | Token inválido o expirado; renovar y reconectar         |
| `user_not_synced`        | El token es válido pero el usuario no existe en la base |
| `conversation_not_found` | No pertenece a la conversación                          |
| `empty_message`          | El cuerpo está vacío tras recortar espacios             |
| `invalid_payload`        | Cuerpo fuera de contrato                                |
| `unknown_event`          | Evento no reconocido                                    |
| `slow_down`              | (Device flow, REST) polling demasiado rápido            |

## Presencia

Al abrir la primera conexión de un usuario se emite `presence.update` con `online` a todos los
conectados; al cerrar la última, `offline`. Con varias instancias del backend este estado debe
compartirse por Redis.
