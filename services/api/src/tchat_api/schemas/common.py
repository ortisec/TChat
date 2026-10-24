"""Esquemas compartidos por los endpoints de sistema."""

from typing import Literal

from pydantic import BaseModel, Field


class HealthResponse(BaseModel):
    status: Literal["ok"] = "ok"
    version: str = Field(description="Versión de la API")


class ReadinessResponse(BaseModel):
    status: Literal["ok", "degraded"]
    database: Literal["up", "down"]


class MetaResponse(BaseModel):
    name: str
    version: str
    environment: str
    firebase_project_id: str
