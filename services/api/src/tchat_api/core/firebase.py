"""Inicialización diferida del SDK de Firebase."""

import json

import firebase_admin
from firebase_admin import credentials

from tchat_api.core.config import settings

_app: firebase_admin.App | None = None


def get_firebase_app() -> firebase_admin.App:
    """Inicializa Firebase una sola vez.

    La verificación de ID tokens solo necesita el project id: las firmas se validan con los
    certificados públicos de Google. La service account se usa si está configurada, por ejemplo
    para futuras operaciones administrativas.
    """
    global _app

    if _app is not None:
        return _app

    if settings.firebase_service_account_json:
        info = json.loads(settings.firebase_service_account_json)
        _app = firebase_admin.initialize_app(credentials.Certificate(info))
    else:
        _app = firebase_admin.initialize_app(options={"projectId": settings.firebase_project_id})

    return _app
