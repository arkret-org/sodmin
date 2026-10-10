#!/usr/bin/env python3
"""Require the real Coauth public signing keys to survive an ordinary restart."""

import json
import pathlib
import subprocess
import urllib.request


def public_keys() -> dict:
    with urllib.request.urlopen("http://127.0.0.1:57080/oauth/keys.json", timeout=10) as response:
        if response.status != 200:
            raise SystemExit(f"Coauth JWKS returned HTTP {response.status}")
        document = json.load(response)
    if (
        not isinstance(document, dict)
        or not isinstance(document.get("keys"), list)
        or not document["keys"]
    ):
        raise SystemExit("Coauth did not expose a nonempty public signing key set")
    private_fields = {"d", "p", "q", "dp", "dq", "qi", "oth", "k"}
    if any(not isinstance(key, dict) or private_fields.intersection(key) for key in document["keys"]):
        raise SystemExit("Coauth JWKS contains malformed or private key material")
    # JWKS ordering carries no meaning; compare every public key field exactly.
    document["keys"] = sorted(document["keys"], key=lambda key: json.dumps(key, sort_keys=True))
    return document


def main() -> None:
    compose_file = pathlib.Path(__file__).resolve().parent / "docker-compose.example-stack.yaml"
    compose = ["docker", "compose", "-f", str(compose_file)]
    before = public_keys()
    subprocess.run([*compose, "restart", "coauth"], check=True)
    subprocess.run([*compose, "up", "-d", "--wait", "coauth"], check=True)
    if public_keys() != before:
        raise SystemExit("Coauth public signing keys changed across an ordinary restart")
    print("[example-stack/keys] ordinary Coauth restart retained its public signing keys")


if __name__ == "__main__":
    main()
