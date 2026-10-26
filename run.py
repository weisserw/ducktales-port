#!/usr/bin/env python3
import os, runpy, sys
from pathlib import Path
workspace = Path(__file__).resolve().parent
toolkit = Path(os.environ.get("ROMATLAS_ROOT", workspace.parent / "romatlas")).resolve()
sys.path.insert(0, str(toolkit / "scripts"))
if len(sys.argv) < 2:
    sys.argv.append("instructions")
if sys.argv[1] in {"web", "web-build"}:
    import argparse
    import json
    import port_web
    sys.path.insert(0, str(workspace / "scripts"))
    from browser_page import install, server

    parser = argparse.ArgumentParser(description="Build or serve this port's browser page")
    parser.add_argument("command", choices=["web", "web-build"])
    parser.add_argument("--host", default="0.0.0.0")
    parser.add_argument("--port", type=int, default=8000)
    parser.add_argument("--assets", type=Path)
    parser.add_argument("--rom", type=Path)
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    bundle = port_web.build(workspace)
    install(bundle)
    if args.command == "web-build":
        print(json.dumps({"ok": True, "bundle": str(bundle)}))
    else:
        manifest = json.loads((workspace / "workspace.json").read_text())
        rom = args.rom or workspace / manifest["rom"]
        with server(port_web, rom, bundle, args.assets, args.port, args.host) as http:
            address = http.server_address[0]
            browser_host = "127.0.0.1" if address == "0.0.0.0" else address
            print(json.dumps({"ok": True, "host": address,
                              "url": f"http://{browser_host}:{http.server_port}/",
                              "bundle": str(bundle)}), flush=True)
            try:
                http.serve_forever()
            except KeyboardInterrupt:
                pass
    raise SystemExit(0)
sys.argv = [str(toolkit / "scripts/port.py"), sys.argv[1], str(workspace), *sys.argv[2:]]
runpy.run_path(str(toolkit / "scripts/port.py"), run_name="__main__")
