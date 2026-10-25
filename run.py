#!/usr/bin/env python3
import os, runpy, sys
from pathlib import Path
workspace = Path(__file__).resolve().parent
toolkit = Path(os.environ.get("ROMATLAS_ROOT", workspace.parent / "romatlas")).resolve()
sys.path.insert(0, str(toolkit / "scripts"))
if len(sys.argv) < 2:
    sys.argv.append("instructions")
sys.argv = [str(toolkit / "scripts/port.py"), sys.argv[1], str(workspace), *sys.argv[2:]]
runpy.run_path(str(toolkit / "scripts/port.py"), run_name="__main__")
