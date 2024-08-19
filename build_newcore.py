"""Utility for building a local copy of the native extension."""

import shutil
import subprocess
from pathlib import Path
import sys

LIB_NAME = "overviewer_core_new"


def main() -> None:
    base_dir = Path(__file__).parent
    build_dir = base_dir / "build_new"
    build_dir.mkdir(parents=True, exist_ok=True)

    # Build extension
    print("Building extension")
    proc = subprocess.run(
        args=[
            "cargo",
            "build",
            "--release",
            "--config",
            'build.target-dir = "{0}"'.format(build_dir.as_posix()),
        ],
        cwd=base_dir,
    )
    if proc.returncode != 0:
        print("Build failed")
        exit(1)

    if sys.platform == "win32":
        src_name = f"{LIB_NAME}.dll"
        dst_name = f"{LIB_NAME}.pyd"
    else:
        src_name = f"lib{LIB_NAME}.so"
        dst_name = f"{LIB_NAME}.so"

    src_path = build_dir / "release" / src_name
    dst_path = base_dir / "overviewer_core" / dst_name
    if not src_path.exists():
        print("Error: Build artifact not found")
        exit(1)

    shutil.copyfile(src_path, dst_path)
    print("Built")


if __name__ == "__main__":
    main()