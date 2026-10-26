import os
"""Exercise the persistent save slot against the real Wasm in headless Chromium."""
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from threading import Thread
import sys
from playwright.sync_api import sync_playwright

class Handler(SimpleHTTPRequestHandler):
    def log_message(self, *_args):
        pass

server = ThreadingHTTPServer(('127.0.0.1', 0), partial(Handler, directory=os.environ.get('DUCKTALES_WEB_DIST',str(Path(__file__).resolve().parents[1] / 'web-dist'))))
thread = Thread(target=server.serve_forever, daemon=True)
thread.start()
try:
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True, args=['--no-sandbox'])
        page = browser.new_page()
        # Reproduce plain HTTP LAN origins, where Web Crypto is unavailable.
        page.add_init_script("Object.defineProperty(window,'crypto',{value:undefined,configurable:true})")
        errors = []
        page.on('pageerror', lambda error: errors.append(str(error)))
        page.goto(sys.argv[1] if len(sys.argv) > 1 else f'http://127.0.0.1:{server.server_port}/')
        ready = "!document.getElementById('start').disabled"
        page.wait_for_function(ready)
        assert page.locator('#restore-state').is_disabled()
        page.click('#start')
        page.wait_for_function("Number(document.getElementById('screen').dataset.ticks)>15")
        page.click('#pause')
        pixels = "document.getElementById('screen').toDataURL()"
        saved_pixels = page.evaluate(pixels)
        saved_ticks = page.locator('#screen').get_attribute('data-ticks')
        page.click('#save-state')
        page.wait_for_function("document.getElementById('state-status').textContent==='State saved.' && !document.getElementById('save-state').disabled")
        key = page.evaluate("Object.keys(localStorage).find(k=>k.includes(':save-state:'))")
        original = page.evaluate('key=>localStorage.getItem(key)', key)
        assert original
        page.click('#start')
        page.wait_for_function(f"Number(document.getElementById('screen').dataset.ticks)>{int(saved_ticks)+15}")
        page.click('#restore-state')
        page.wait_for_function("document.getElementById('game-message').textContent.startsWith('State restored')")
        assert page.locator('#game-overlay').is_visible()
        assert page.locator('#status').is_hidden()
        assert page.locator('#state-status').inner_text() == ''
        assert page.evaluate(pixels) == saved_pixels
        assert page.locator('#screen').get_attribute('data-ticks') == saved_ticks
        # Restore remains available across a full page reload.
        page.click('#reset')
        page.wait_for_function(ready)
        assert page.locator('#restore-state').is_enabled()
        page.click('#restore-state')
        page.wait_for_function("document.getElementById('game-message').textContent.startsWith('State restored')")
        assert page.locator('#game-overlay').is_visible()
        assert page.locator('#status').is_hidden()
        assert page.locator('#state-status').inner_text() == ''
        assert page.evaluate(pixels) == saved_pixels
        page.click('#start')
        page.wait_for_function(f"Number(document.getElementById('screen').dataset.ticks)>{int(saved_ticks)+5}")
        page.click('#pause'); page.click('#save-state')
        page.wait_for_function("!document.getElementById('save-state').disabled")
        replacement = page.evaluate('key=>localStorage.getItem(key)', key)
        assert replacement != original
        assert page.evaluate("Object.keys(localStorage).filter(k=>k.includes(':save-state:')).length") == 1
        before = page.evaluate(pixels)
        # Incompatible saves and corrupted journals must not replace the game.
        for bad in ['{"version":999}', '{bad json']:
            page.evaluate('([key,value])=>localStorage.setItem(key,value)', [key,bad])
            page.click('#restore-state')
            page.wait_for_function("document.getElementById('state-status').textContent.startsWith('Restore failed')")
            assert page.evaluate(pixels) == before
        page.evaluate('([key,value])=>localStorage.setItem(key,value)', [key,replacement])
        page.evaluate("() => {window.originalSetItem=Storage.prototype.setItem;Storage.prototype.setItem=function(){throw new DOMException('test quota','QuotaExceededError')};}")
        page.click('#save-state')
        page.wait_for_function("document.getElementById('error').textContent.includes('Save state failed')")
        page.evaluate('() => {Storage.prototype.setItem=window.originalSetItem;}')
        assert page.evaluate('key=>localStorage.getItem(key)',key) == replacement
        assert not errors, errors
        browser.close()
        print('PASS: real Wasm frame/tick restore, persistence across Reset, slot overwrite, incompatible/corrupt rejection, quota failure preserves prior save')
finally:
    server.shutdown(); server.server_close(); thread.join()
