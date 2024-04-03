#!/usr/bin/python3.11
import json
import os
import sys
import tempfile
from pathlib import Path

try:
    import tomllib  # requires Python3.11+
except ImportError:
    print("Are you sure you're using Python 3.11 or newer?", file=sys.stderr)
    exit(1)

# see `rclone config` -> garage-s3-ntfy-log
RCLONE_ENDPOINT = "garage-s3-ntfy-log"
BINARY_NAME = "ntfy-log"
BUCKET_NAME = "ntfy-log"
BASE_DOWNLOAD_URL = "https://download.s3.su6.nl"


def is_empty(f: Path) -> bool:
    return f.stat().st_size == 0


def read_cargo():
    with open("Cargo.toml", "rb") as f:
        data = tomllib.load(f)

    return data["package"]


def main():
    targets = [
        "x86_64-unknown-linux-gnu",
        "aarch64-unknown-linux-gnu",
    ]
    archs = [
        _.split("-")[0]
        for _
        in targets
    ]

    # for target in targets:
    #     exit_code = os.system(f"cargo build --release --target {target}")
    #     if exit_code:
    #         exit(exit_code)

    # # separate loop to make sure none of the targets crashed
    # for arch in archs:
    #     binary_path = f"./target/{target}/release/{BINARY_NAME}"
    #     os.system(
    #         f"rclone copy --progress {binary_path} {RCLONE_ENDPOINT}:{BUCKET_NAME}/{arch}/"
    #     )

    # update manifest:
    with tempfile.NamedTemporaryFile(suffix="build-release.json") as f:
        os.system(
            f"rclone copyto --progress {RCLONE_ENDPOINT}:{BUCKET_NAME}/index.json {f.name}"
        )

        f_path = Path(f.name)
        current_contents = (
            json.loads(f_path.read_text())
            if (f_path.exists() and not is_empty(f_path))
            else {}
        )

        current_contents[BINARY_NAME] = read_cargo() | {
            "downloads": {_: f"{BASE_DOWNLOAD_URL}/{_}/{BINARY_NAME}" for _ in archs}
        }
        f_path.write_text(json.dumps(current_contents))

        os.system(
            f"rclone copyto --progress {f.name} {RCLONE_ENDPOINT}:{BUCKET_NAME}/index.json"
        )
        # update dmanifest at download.s3.su6.nl


if __name__ == "__main__":
    main()
