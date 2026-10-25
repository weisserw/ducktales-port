"""Bounded original comparisons, isolated from native gameplay regressions.

Align stable screens and player/camera landmarks, never whole-run frame counts.
Only read reference memory for named visual/audio landmarks; no writes.
"""
import hashlib
import json
from pathlib import Path
import re
import struct
import wave
from native_debug_client import PortClient

ROOT = Path('/root/ducktales')
SYMBOLS = {n:int(a,16) for n,a in re.findall(r'^(\w+) = \$([0-9A-Fa-f]+)$', (ROOT/'reference/source/symbols.inc').read_text(), re.M)}

def save(path, value):
    path.write_text(json.dumps(value, indent=2)+'\n')

class Sanity:
    def __init__(self, run):
        self.run=run
        self.n=run.stack.enter_context(PortClient(['python3',str(ROOT/'run.py'),'debug'],seconds=900,max_requests=5000,evidence=run.evidence/'native'))
        self.r=run.stack.enter_context(PortClient(['/root/romatlas/target/release/romatlas','serve','--platform','nes','--rom',str(ROOT/'input/game.rom')],seconds=900,max_requests=5000,evidence=run.evidence/'original'))
        run.clients.update(native=self.n, original=self.r)
        self.r.request('emulator.start')
        save(run.evidence/'identities.json',{'native':self.n.request('api.describe'),'original':self.r.request('api.describe'),'scenario_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()})
    def state(self):return self.n.request('port.state')['game']
    def mem(self,name,count=1):return self.r.request('emulator.memory',{'address':SYMBOLS[name],'length':count})['bytes']
    def step(self,side,ticks):
        if side=='n':
            result=self.n.request('port.advance',{'ticks':ticks,'timeout_ms':300000})
            assert result['complete'] and result['completed']==ticks
        else:
            before=self.r.request('emulator.state')['frames']
            result=self.r.request('emulator.step',{'unit':'frame','count':ticks,'max_operations':100000000,'timeout_ms':300000})
            assert result['frames']-before==ticks
    def wait(self,side,predicate,bound):
        for _ in range(bound):
            if predicate():return
            self.step(side,1)
        raise AssertionError('bounded sanity landmark not reached')
    def inputs(self,actions):
        self.n.request('port.input',{'actions':actions})
        self.r.request('emulator.input',{'port':1,'buttons':actions})
    def edge(self,actions):
        self.inputs([])
        for s in 'nr':self.step(s,1)
        self.inputs(actions)
        for s in 'nr':self.step(s,1)
        self.inputs([])
    def capture(self,name):
        self.n.frame(name+'.ppm')
        self.r.request('emulator.framebuffer',{'path':str(self.run.evidence/'original'/(name+'.ppm'))})
        save(self.run.evidence/(name+'-state.json'),self.state())
    def audio(self,name,ticks=90):
        self.n.request('port.audio.clear');self.r.request('emulator.audio.clear')
        for side in 'nr':self.step(side,ticks)
        meta=self.n.request('port.audio',{'offset':0,'limit':0});assert meta['dropped_frames']==0
        samples=[]
        for off in range(0,meta['frames'],4096):samples.extend(self.n.request('port.audio',{'offset':off,'limit':min(4096,meta['frames']-off)})['samples'])
        assert samples and max(map(abs,samples))>0.01,'missing native audio'
        with wave.open(str(self.run.evidence/'native'/(name+'.wav')),'wb') as w:
            w.setparams((2,2,48000,0,'NONE','not compressed'))
            w.writeframes(struct.pack('<'+'h'*len(samples),*[round(max(-1,min(1,x))*32767) for x in samples]))
        original=self.r.request('emulator.audio.export',{'path':str(self.run.evidence/'original'/(name+'.wav'))})
        assert original['dropped_stereo_frames']==0
        save(self.run.evidence/(name+'-audio.json'),{'native':meta,'original':original,'peak':max(map(abs,samples))})
    def title(self):
        self.wait('n',lambda:self.state()['transition']['menu']=='Title(Input)',120)
        self.wait('r',lambda:self.mem('TitleOption')==[1] and self.mem('ObjectStateFlags',2)==[128,128],200)
        self.inputs([])
        for side in 'nr':self.step(side,1)
    def return_address(self):
        sp=self.mem('SchedulerSavedStackOrEntry')[0]
        stack=self.r.request('emulator.memory',{'address':0x100+sp+1,'length':4})['bytes']
        return stack[2]+256*stack[3]
    def amazon(self):
        self.edge(['start'])
        self.wait('n',lambda:self.state()['transition']['menu']=='LandInput',180)
        self.wait('r',lambda:self.return_address()==0xD791,180)
        self.edge(['a'])
        self.wait('n',lambda:self.state()['mode']=='gameplay' and self.state()['transition']['foreground']=='Yield',350)
        self.wait('r',lambda:self.mem('GameplayDisplayMode')==[0x40] and self.return_address()==0xDC28,350)
        self.inputs([])
        for side in 'nr':self.step(side,16)

def run(context,entry):
    with context.phase('setup'):
        game=Sanity(context);game.title()
    with context.phase('execution'):
        if entry['id']=='fidelity-title':
            def check():
                game.capture('title-normal')
                assert game.state()['actors'][1]['position']['world_x']==0x50
                game.edge(['right'])
                assert game.state()['actors'][1]['position']['world_x']==0x90
                assert game.mem('TitleOption')==[2]
                game.capture('title-difficult');game.audio('title-music')
            context.check('readable title and option movement with active music',check)
        elif entry['id']=='fidelity-amazon-motion':
            def check():
                game.amazon();game.capture('amazon-entry')
                before=game.state()['world']['camera']['x']
                game.inputs(['right','a'])
                for side in 'nr':game.step(side,28)
                game.inputs(['right','down','b'])
                for side in 'nr':game.step(side,80)
                game.capture('amazon-traverse')
                after=game.state()['world']['camera']['x']
                assert after>before,{'before':before,'after':after}
                assert game.mem('CameraX')[0]+256*game.mem('CameraXHi')[0]>0
                game.inputs([]);game.audio('amazon-music')
            context.check('terrain scroll and animated player during short traversal',check)
        else:raise ValueError(entry['id'])
    return {'presentation_review_required':True,'criteria':entry['review'],'alignment':'stable semantic screen, then bounded representative motion; hardware loading lag is not compared'}
