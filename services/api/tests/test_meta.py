"""Metadatos públicos."""

from httpx import AsyncClient


async def test_meta(client: AsyncClient) -> None:
    response = await client.get("/api/v1/meta")

    assert response.status_code == 200
    body = response.json()
    assert body["name"] == "TChat API"
    assert body["environment"] == "development"
    assert body["firebase_project_id"] == "tchat-c20ce"
