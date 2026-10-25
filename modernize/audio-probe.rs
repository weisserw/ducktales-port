//! Direct audio-owner comparison, not an original-emulator or device test.
//! All56 original headers run for240 NTSC ticks; two extra cases exercise
//! preemption/control commands. Actual register writes feed the shared APU.
use ducktales_port::{audio,config};
use serde_json::json;
use std::{fs::{self,File},io::{BufWriter,Write},path::Path};
fn main(){
 let args:Vec<_>=std::env::args().collect();let rom=fs::read(&args[1]).unwrap();let out=Path::new(&args[2]);fs::create_dir_all(out).unwrap();
 let catalog=audio::Catalog::from_rom(&rom).unwrap();let mut records=BufWriter::new(File::create(out.join("observations.jsonl")).unwrap());
 let mut count=0;let mut samples_count=0;
 for case in 0..58u8 {
  let mut d=audio::Driver::default();let mut q=audio::CommandQueue::default();let mut synth=audio::Synthesizer::default();
  let mut pcm=BufWriter::new(File::create(out.join(format!("case-{case:02}.f32le"))).unwrap());let mut fraction=0u64;
  q.submit(if case<56{case}else if case==56{0}else{15});
  for tick in 0..240u16 {
   if case>=56 {match tick {60=>{q.submit(if case==56{16}else{55});},90=>{q.submit(0xfd);},120=>{q.submit(0xfc);},150=>{q.submit(0xfe);},180=>{q.submit(0xff);},_=>()}}
   let mut reads=Vec::new();
   let writes=d.interrupt_with_alias(&mut q,&catalog,((tick*7)as u8)^case,&mut |address|{let value=(address as u8)^((address>>8)as u8)^case;reads.push((address,value));value});
   synth.apply(&writes);
   fraction+=48_000u64*u64::from(config::RATE.denominator);let frames=fraction/u64::from(config::RATE.numerator);fraction%=u64::from(config::RATE.numerator);
   let mut samples=vec![0.0;frames as usize];synth.render(&mut samples,48_000);for value in samples {assert!(value.is_finite());pcm.write_all(&value.to_le_bytes()).unwrap();samples_count+=1;}
   writeln!(records,"{}",json!({"case":case,"tick":tick,"driver":format!("{d:?}"),"queue":q.slots.to_vec(),"producer":q.producer,"consumer":q.consumer,"writes":writes.iter().map(|w|(w.address,w.value)).collect::<Vec<_>>(),"alias_reads":reads,"clocks":synth.elapsed_clocks,"fraction":synth.sample_clock_fraction.to_bits()})).unwrap();count+=1;
  }
 }
 println!("{count} complete audio frames; {samples_count} mono PCM samples across58 cases");
}
