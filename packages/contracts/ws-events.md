# Protocolo WebSocket (borrador)

> Estado: borrador del Paso 2. Se implementa en el Paso 4 y se valida en el Paso 5.

- URL: `wss://ws.tchat.site/ws`
- Autenticación: cabecera `Authorization: Bearer <id_token>` en el handshake.
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

| `type`         | payload                                | Descripción                            |
| -------------- | -------------------------------------- | -------------------------------------- |
| `ping`         | `{}`                                   | Keepalive; el servidor responde `pong` |
| `message.send` | `conversation_id`, `body`, `client_id` | Envía un mensaje                       |
| `message.read` | `conversation_id`, `up_to_message_id`  | Marca como leído                       |
| `typing.start` | `conversation_id`                      | El usuario está escribiendo            |
| `typing.stop`  | `conversation_id`                      | Dejó de escribir                       |

## Servidor → cliente

| `type`            | payload                                 | Descripción                |
| ----------------- | --------------------------------------- | -------------------------- |
| `pong`            | `{}`                                    | Respuesta a `ping`         |
| `message.new`     | `message`                               | Mensaje entrante           |
| `message.ack`     | `client_id`, `message_id`, `created_at` | Confirmación de envío      |
| `presence.update` | `user_id`, `status`                     | `online` / `offline`       |
| `error`           | `code`, `message`, `ref`                | Error asociado a un evento |

## Errores previstos

| `code`            | Significado                                                       |
| ----------------- | ----------------------------------------------------------------- |
| `unauthorized`    | Token inválido o expirado; el cliente debe renovarlo y reconectar |
| `forbidden`       | No pertenece a la conversación                                    |
| `invalid_payload` | Cuerpo fuera de contrato                                          |
| `rate_limited`    | Demasiados eventos por ventana                                    |
