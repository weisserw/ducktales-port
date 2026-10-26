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
assert.equal(game.width(),341);assert.equal(game.height(),240);assert.equal(game.display_aspect(),16/9);
const displayedPixelAspect=game.display_aspect()*game.height()/game.width();
const originalPixelAspect=(4/3)*240/256;
assert.ok(Math.abs(displayedPixelAspect-originalPixelAspect)<.002,'original display pixel proportions');
const advance=(count,...keys)=>{for(let i=0;i<count;i++){game.advance(17,physical(...keys));game.audio();}};
advance(180);advance(1,'Return');advance(180);advance(1,'Z');advance(360);
const saved=game.snapshot_json();const machine=JSON.parse(JSON.parse(saved).game);
assert.equal(machine.services.video.wide_gameplay,true);
assert.ok(machine.services.video.wide_terrain.rooms.length>=2);
assert.ok(machine.services.video.committed_wide_sprites.length>0);
const pixels=game.pixels();
// Centered, original-aspect HUD and a real map at both gameplay edges.
assert.deepEqual([...pixels.slice(0,4)],[0,0,0,255]);
for(const x of [0,game.width()-1]) {
  const colors=new Set();for(let y=60;y<240;y++)colors.add([...pixels.slice((y*game.width()+x)*4,(y*game.width()+x+1)*4)].join(','));
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
console.log(`PASS: real 341x240 Wasm gameplay, 16:9 display, terrain at both edges, committed sprites and 90 deterministic frame/audio continuations (${((performance.now()-begin)/180).toFixed(1)} ms/update)`);
// Reproduce Scrooge entering the HUD band using an otherwise real gameplay
// snapshot. Let the normal compose/NMI path commit his new sprite positions.
const raised=create();
const high=JSON.parse(saved),highMachine=JSON.parse(high.game);
highMachine.core.actors[0].position.screen_y=32;
highMachine.core.actors[0].position.y_fraction=0;
highMachine.core.actors[0].vertical_velocity=0;
highMachine.core.player.action='Airborne';
highMachine.core.player.protection=0;
highMachine.core.player.special_mode=0;
high.game=JSON.stringify(highMachine);
raised.restore_snapshot_json(JSON.stringify(high));
for(let i=0;i<4;i++){raised.advance(17,physical());raised.audio();}
const raisedSave=JSON.parse(raised.snapshot_json()),raisedMachine=JSON.parse(raisedSave.game);
assert.ok(raisedMachine.core.actors[0].position.screen_y<68);
assert.equal(raisedMachine.services.video.committed_wide_gameplay,true);
assert.ok(raisedMachine.services.video.committed_wide_sprites.some(sprite=>sprite.y<44));
// Removing only native sprite drawing gives the exact HUD/background baseline.
const hidden=create();
raisedMachine.services.video.committed_wide_sprites=[];
raisedSave.game=JSON.stringify(raisedMachine);
hidden.restore_snapshot_json(JSON.stringify(raisedSave));
const visible=raised.pixels(),background=hidden.pixels();
let hudSpritePixels=0;
for(let y=0;y<44;y++)for(let x=0;x<raised.width();x++) {
 const at=(y*raised.width()+x)*4;
 if(visible[at]!==background[at] || visible[at+1]!==background[at+1] || visible[at+2]!==background[at+2])hudSpritePixels++;
}
assert.ok(hudSpritePixels>0,'Scrooge must remain visible over the HUD');
console.log(`PASS: real Wasm Scrooge sprite remains visible above the HUD split (${hudSpritePixels} foreground pixels)`);
raised.free();hidden.free();game.free();restored.free();
