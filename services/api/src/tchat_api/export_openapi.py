"""Exporta el contrato OpenAPI a packages/contracts/openapi.json."""

import json
from pathlib import Path

from tchat_api.main import app


def main() -> None:
    repo_root = Path(__file__).resolve().parents[4]
    target = repo_root / "packages" / "contracts" / "openapi.json"
    target.parent.mkdir(parents=True, exist_ok=True)

    spec = app.openapi()
    content = json.dumps(spec, indent=2, ensure_ascii=False)
    target.write_text(f"{content}\n", encoding="utf-8")
    print(f"OpenAPI escrito en {target}")


if __name__ == "__main__":
    main()
