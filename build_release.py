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

    # target - arch tuples
    targets = [
        (target, target.split("-")[0])
        for target in (
            "x86_64-unknown-linux-gnu",
            "aarch64-unknown-linux-gnu",
        )
    ]

    for target, _ in targets:
        exit_code = os.system(f"cargo build --release --target {target}")
        if exit_code:
            exit(exit_code)

    # separate loop to make sure none of the targets crashed
    for target, arch in targets:
        binary_path = f"./target/{target}/release/{BINARY_NAME}"
        os.system(
            f"rclone copy --progress {binary_path} {RCLONE_ENDPOINT}:{BUCKET_NAME}/{arch}/"
        )

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
            "downloads": {arch: f"{BASE_DOWNLOAD_URL}/{arch}/{BINARY_NAME}" for (_, arch) in targets}
        }
        f_path.write_text(json.dumps(current_contents))

        os.system(
            f"rclone copyto --progress {f.name} {RCLONE_ENDPOINT}:{BUCKET_NAME}/index.json"
        )
        # updated manifest at download.s3.su6.nl

        # sudo wget https://download.s3.su6.nl/$(uname -m)/ntfy-log -O /usr/local/bin/ntfy-log

if __name__ == "__main__":
    main()
