"""Configuración del servicio."""

from functools import lru_cache

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """Variables de entorno de la API.

    Los valores por defecto permiten arrancar en local sin configurar nada.
    En producción todo se inyecta por entorno.
    """

    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        extra="ignore",
    )

    app_name: str = "TChat API"
    environment: str = "development"
    debug: bool = False
    database_url: str = "postgresql+asyncpg://tchat:tchat@localhost:5432/tchat"
    cors_origins: str = "http://localhost:4321,http://localhost:4322,http://localhost:4323"
    firebase_project_id: str = "tchat-c20ce"
    firebase_service_account_json: str | None = None
    public_account_url: str = "http://localhost:4322"
    auth_jwt_secret: str = "dev-secret-change-me-32-characters-long"
    auth_jwt_issuer: str = "tchat"
    access_token_ttl_seconds: int = 900
    device_code_ttl_seconds: int = 600
    device_poll_interval_seconds: int = 5

    @property
    def cors_origin_list(self) -> list[str]:
        """Convierte la lista separada por comas en una lista limpia."""
        return [origin.strip() for origin in self.cors_origins.split(",") if origin.strip()]


@lru_cache
def get_settings() -> Settings:
    return Settings()


settings = get_settings()
