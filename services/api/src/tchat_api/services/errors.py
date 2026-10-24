"""Errores de dominio que la capa HTTP traduce a respuestas."""


class ServiceError(Exception):
    """Error de negocio con un código estable para el cliente."""

    status_code = 400

    def __init__(self, code: str, *, status_code: int | None = None) -> None:
        super().__init__(code)
        self.code = code
        if status_code is not None:
            self.status_code = status_code


class NotFoundError(ServiceError):
    status_code = 404


class ConflictError(ServiceError):
    status_code = 409


class ForbiddenError(ServiceError):
    status_code = 403
