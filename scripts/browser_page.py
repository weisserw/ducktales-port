"""This port's browser page, shared by local serving and Pages packaging."""
from pathlib import Path
import shutil

WORKSPACE = Path(__file__).resolve().parents[1]


def install(bundle, *, relative=False):
    bundle = Path(bundle)
    for name in ('index.html', 'shell.js', 'save-state.mjs'):
        shutil.copyfile(WORKSPACE / 'web' / name, bundle / name)
    shutil.copyfile(WORKSPACE / 'scripts/browser-gamepads.mjs', bundle / 'browser-gamepads.mjs')
    if relative:
        for name, replacements in {
            'index.html': [('src="/shell.js"', 'src="./shell.js"')],
            'shell.js': [("checkedFetch('/config.json')", "checkedFetch('./config.json')"),
                         ("checkedFetch('/rom')", "checkedFetch('./rom')")],
        }.items():
            path = bundle / name
            text = path.read_text()
            for old, new in replacements:
                if text.count(old) != 1:
                    raise ValueError(f'Expected exactly one {old!r} in {name}; review page changes')
                text = text.replace(old, new)
            path.write_text(text)


def server(port_web, rom, bundle, assets, port, host):
    http = port_web.server(WORKSPACE, rom, bundle, assets, port, host)
    original = http.RequestHandlerClass
    modules = {f'/{name}': (Path(bundle) / name).read_bytes()
               for name in ('save-state.mjs', 'browser-gamepads.mjs')}

    class Handler(original):
        def do_GET(self):
            from urllib.parse import urlsplit
            data = modules.get(urlsplit(self.path).path)
            if data is None:
                return super().do_GET()
            self.send_response(200)
            self.send_header('Content-Type', 'text/javascript')
            self.send_header('Content-Length', str(len(data)))
            self.send_header('Cache-Control', 'no-store')
            self.send_header('X-Content-Type-Options', 'nosniff')
            self.end_headers()
            try:
                self.wfile.write(data)
            except (BrokenPipeError, ConnectionResetError):
                pass

    http.RequestHandlerClass = Handler
    return http
