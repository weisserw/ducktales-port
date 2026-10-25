#!/usr/bin/env python3
"""Build the static GitHub Pages bundle, including the configured ROM."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys

workspace = Path(__file__).resolve().parents[1]
toolkit = Path(os.environ.get("ROMATLAS_ROOT", workspace.parent / "romatlas")).resolve()
sys.path.insert(0, str(toolkit / "scripts"))
import port_web

manifest = json.loads((workspace / "workspace.json").read_text())
rom = workspace / manifest["rom"]
rom_hash = hashlib.sha256(rom.read_bytes()).hexdigest()
if rom_hash != manifest["sha256"]:
    raise SystemExit("ROM SHA-256 does not match workspace.json")
bundle = port_web.build(workspace)
# The shared development server uses root routes; project Pages uses a subpath.
for filename, replacements in {
    "index.html": [('src="/shell.js"', 'src="./shell.js"')],
    "shell.js": [("checkedFetch('/config.json')", "checkedFetch('./config.json')"),
                 ("checkedFetch('/rom')", "checkedFetch('./rom')")],
}.items():
    path = bundle / filename
    text = path.read_text()
    for old, new in replacements:
        if text.count(old) != 1:
            raise SystemExit(f"Expected exactly one {old!r} in {filename}; review toolkit shell changes")
        text = text.replace(old, new)
    path.write_text(text)
# Keep controller compatibility changes in this repository so the pinned toolkit
# checkout in CI produces the same browser frontend as local builds.
shell = bundle / "shell.js"
text = shell.read_text()
start = text.index("function pads() {")
end = text.index("function physical()", start)
text = text[:start] + "function pads() { return gamepads(navigator); }\n" + text[end:]
start = text.index("      p.buttons.forEach")
end = text.index("\n    }", start)
text = text[:start] + "      Object.assign(state, padState(p)); if(index===0)state.keys=[...keys];" + text[end:]
start = text.index("    if(capture){const state=physical()")
end = text.index("\n    if(game", start)
text = text[:start] + "    if(capture && pollCapture){const binding=pollCapture();if(binding)bind(binding);}" + text[end:]
text = text.replace("capture=action;button.textContent", "capture=action;pollCapture=capturePadBinding(pads);button.textContent")
text = "import {gamepads, padState, capturePadBinding} from './browser-gamepads.mjs';\nlet pollCapture;\n" + text
shell.write_text(text)
shutil.copyfile(workspace / "scripts/browser-gamepads.mjs", bundle / "browser-gamepads.mjs")
shutil.copyfile(rom, bundle / "rom")
(bundle / "config.json").write_text(json.dumps({
    "title": "DuckTales", "rom_sha256": rom_hash,
    "assets": {"enabled": False, "manifests": {}, "files": {}},
}) + "\n")
(bundle / ".nojekyll").touch()
print(f"Static site ready: {bundle}")
