import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
const bundle=path.resolve(process.env.DUCKTALES_WEB_DIST || 'web-dist');
const {default:init,BrowserGame,BrowserAssets}=await import(pathToFileURL(path.join(bundle,'game.js')));
await init({module_or_path:fs.readFileSync(path.join(bundle,'game_bg.wasm'))});
const create=()=>new BrowserGame(new Uint8Array(fs.readFileSync(path.join(bundle,'rom'))),new BrowserAssets('{"enabled":false,"manifests":{},"files":{}}'),48000);
const physical=(...keys)=>JSON.stringify(Array.from({length:4},(_,i)=>({keys:i===0?keys:[],buttons:[],axes:{}})));
const game=create();
assert.equal(game.width(),720);assert.equal(game.height(),240);assert.equal(game.display_aspect(),16/9);
const advance=(count,...keys)=>{for(let i=0;i<count;i++){game.advance(17,physical(...keys));game.audio();}};
advance(180);advance(1,'Return');advance(180);advance(1,'Z');advance(360);
const saved=game.snapshot_json();const machine=JSON.parse(JSON.parse(saved).game);
assert.equal(machine.services.video.wide_gameplay,true);
assert.ok(machine.services.video.wide_terrain.rooms.length>=3);
assert.ok(machine.services.video.committed_wide_sprites.length>0);
const pixels=game.pixels();
// Centered, original-aspect HUD and a real map at both gameplay edges.
assert.deepEqual([...pixels.slice(0,4)],[0,0,0,255]);
for(const x of [0,719]) {
  const colors=new Set();for(let y=60;y<240;y++)colors.add([...pixels.slice((y*720+x)*4,(y*720+x+1)*4)].join(','));
  assert.ok(colors.size>2,`map at gameplay edge ${x}`);
}
const restored=create();restored.restore_snapshot_json(saved);
assert.deepEqual(restored.pixels(),game.pixels());
const begin=performance.now();
for(let i=0;i<90;i++) {
 const input=i<45?physical('ArrowRight','Z'):physical('ArrowLeft','Z');
 assert.equal(restored.advance(17,input),game.advance(17,input));
 assert.deepEqual(restored.pixels(),game.pixels());assert.deepEqual(restored.audio(),game.audio());
}
assert.equal(restored.snapshot_json(),game.snapshot_json());
console.log(`PASS: real 720x240 Wasm gameplay, 16:9 display, terrain at both edges, committed sprites and 90 deterministic frame/audio continuations (${((performance.now()-begin)/180).toFixed(1)} ms/update)`);
game.free();restored.free();
