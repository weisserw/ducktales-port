import os
"""Headless checks for the built page's layout, focus handling and Reset."""
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from threading import Thread
from playwright.sync_api import sync_playwright


class Handler(SimpleHTTPRequestHandler):
    def log_message(self, *_args):
        pass


bundle = Path(os.environ.get('DUCKTALES_WEB_DIST',Path(__file__).resolve().parents[1] / 'web-dist'))
server = ThreadingHTTPServer(('127.0.0.1', 0), partial(Handler, directory=str(bundle)))
thread = Thread(target=server.serve_forever, daemon=True)
thread.start()
try:
    with sync_playwright() as playwright:
        browser = playwright.chromium.launch(headless=True, args=['--no-sandbox'])
        page = browser.new_page(viewport={'width': 1280, 'height': 1000})
        errors = []
        page.on('pageerror', lambda error: errors.append(str(error)))
        page.goto(f'http://127.0.0.1:{server.server_port}/')
        page.wait_for_function("!document.getElementById('start').disabled")
        assert page.locator('h1 + p').inner_text() == ('This is a native recompilation of the original game in Rust, running in WebAssembly. You must click the Start button below to start the game engine and enable audio and input in the browser.')
        assert page.locator('#pause + #reset').inner_text() == 'Reset (F3)'
        assert page.locator('#shortcuts').count() == 0
        for index, name in enumerate(['start','pause','reset','save-state','restore-state','settings'],1):
            assert page.locator('#'+name).inner_text().endswith(f'(F{index})')
        assert page.locator('#screen').evaluate('(canvas) => [canvas.width,canvas.height]') == [341,240]
        box = page.locator('#screen').bounding_box()
        assert abs(box['x'] + box['width'] / 2 - 640) < 2
        assert abs((box['width']/341)/(box['height']/240)-1.25)<.002
        page.click('#start')
        page.wait_for_function("Number(document.getElementById('screen').dataset.ticks)>2")
        for event in ['blur', 'visibilitychange']:
            previous = int(page.locator('#screen').get_attribute('data-ticks'))
            page.evaluate("""event => {
                Object.defineProperty(document,'hasFocus',{value:()=>false,configurable:true});
                Object.defineProperty(document,'hidden',{value:true,configurable:true});
                (event==='blur'?window:document).dispatchEvent(new Event(event));
            }""", event)
            page.wait_for_function(f"Number(document.getElementById('screen').dataset.ticks)>{previous+2}")
            assert page.locator('#status').inner_text() == 'Running'
            page.evaluate('delete document.hasFocus;delete document.hidden')
        page.click('#pause')
        assert page.locator('#game-overlay').is_visible()
        assert page.locator('#game-message').inner_text() == 'Paused — press Start to resume'
        assert page.locator('#status').is_hidden()
        assert page.locator('.edition').count() == 0
        assert page.title() == 'DuckTales'
        assert page.locator('#title').inner_text() == 'DuckTales'
        page.keyboard.press('F1')
        page.wait_for_function("document.getElementById('status').textContent==='Running'")
        page.keyboard.press('F2')
        assert page.locator('#game-overlay').is_visible()
        page.keyboard.press('F6')
        assert page.locator('#preferences').is_visible()
        assert page.locator('#width').count() == 0
        page.locator('#height').fill('360')
        page.locator('#integer_scale').uncheck()
        page.locator('#volume').fill('0.3')
        page.locator('#fullscreen').check()
        page.click('button[type=submit]')
        page.wait_for_function("document.fullscreenElement?.id==='game-panel' && document.getElementById('status').textContent==='Running'")
        box = page.locator('#screen').bounding_box()
        assert abs(box['height']-360)<1 and abs(box['width']-640)<1
        assert page.locator('#game-overlay').is_hidden()
        # Shortcuts remain accessible while page buttons are outside fullscreen.
        page.keyboard.press('F6')
        assert page.locator('#preferences').is_visible()
        page.keyboard.press('F6')
        page.wait_for_function("!document.getElementById('preferences').open && document.getElementById('status').textContent==='Running'")
        page.keyboard.press('F4')
        page.wait_for_function("document.getElementById('state-status').textContent==='State saved.' && !document.getElementById('save-state').disabled")
        page.keyboard.press('F2')
        page.keyboard.press('F5')
        page.wait_for_function("document.getElementById('game-message').textContent.startsWith('State restored')")
        page.keyboard.press('F1')
        page.wait_for_function("document.getElementById('status').textContent==='Running'")
        page.keyboard.press('F6')
        page.locator('#fullscreen').uncheck()
        page.click('button[type=submit]')
        page.wait_for_function("!document.fullscreenElement && document.getElementById('status').textContent==='Running'")
        page.keyboard.press('F3')
        page.wait_for_function("!document.getElementById('start').disabled")
        assert page.locator('#status').inner_text().startswith('Ready')
        page.click('#start')
        page.wait_for_function("Number(document.getElementById('screen').dataset.ticks)>2")
        assert page.locator('#game-overlay').is_hidden()
        page.set_viewport_size({'width': 420, 'height': 900})
        box = page.locator('#screen').bounding_box()
        assert abs(box['x'] + box['width'] / 2 - 210) < 2
        assert box['width'] <= 420
        assert abs(box['width']/box['height']-16/9)<.01
        assert not errors, errors
        browser.close()
        print('PASS: title casing, centered canvas, focus continuity, all toolbar shortcuts, fullscreen settings resume, Pause and Reset/restart')
finally:
    server.shutdown()
    server.server_close()
    thread.join()
