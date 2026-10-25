//! Direct-owner terrain fixture, not a natural game/emulator route. Executes
//! actual world rules and canonical secondary aliases, then samples committed
//! terrain with common source CHR and a fixed review palette.
use ducktales_port::{assets::formats, config, presentation, world};
use romatlas_port::nes::ppu;
use serde_json::json;
use std::{fs, path::Path};
#[cfg(not(modernized))]
fn update_writes(update: &world::TerrainUpload) -> Vec<presentation::VideoWrite> { update.writes.clone() }
#[cfg(modernized)]
fn update_writes(update: &world::TerrainUpdate) -> Vec<presentation::VideoWrite> { presentation::terrain::writes(update) }
#[cfg(not(modernized))]
fn column_writes(result: &world::ColumnResult) -> Option<Vec<presentation::VideoWrite>> { result.upload.as_ref().map(|u|u.writes.clone()) }
#[cfg(modernized)]
fn column_writes(result: &world::ColumnResult) -> Option<Vec<presentation::VideoWrite>> { result.update.as_ref().map(presentation::terrain::column_writes) }
#[cfg(not(modernized))]
fn row_writes(rows: &world::VerticalRows) -> Vec<presentation::VideoWrite> { rows.uploads.clone() }
#[cfg(modernized)]
fn row_writes(rows: &world::VerticalRows) -> Vec<presentation::VideoWrite> { presentation::terrain::writes(&rows.update) }
#[cfg(not(modernized))]
fn burst_writes(burst: &world::Burst) -> Vec<presentation::VideoWrite> { burst.upload.writes.clone() }
#[cfg(modernized)]
fn burst_writes(burst: &world::Burst) -> Vec<presentation::VideoWrite> { presentation::terrain::writes(&burst.update) }
fn pairs(writes: &[presentation::VideoWrite]) -> Vec<(u16,u8)> { writes.iter().map(|w|(w.address,w.value)).collect() }
fn capture(memory:&presentation::VideoMemory,path:&Path){
 let mut bytes=b"P6\n512 240\n255\n".to_vec();let colors=[0x0f,0x30,0x10,0x20];
 for y in 0..240 {for x in 0..512 {
  let view=ppu::Background{chr:&memory.chr,chr_banks:[0,1,2,3,4,5,6,7],nametables:&memory.nametables,mirroring:ppu::Mirroring::Vertical,pattern_table:0x1000,scroll:ppu::Scroll{nametable:(x/256)as u8,..Default::default()}};
  let pixel=view.pixel(x%256,y).unwrap();bytes.extend_from_slice(&ppu::color(&config::NES_RGB,colors[usize::from(pixel&3)],0)[..3]);
 }}fs::write(path,bytes).unwrap();
}
fn main(){
 let args:Vec<_>=std::env::args().collect();let rom=fs::read(&args[1]).unwrap();let out=Path::new(&args[2]);fs::create_dir_all(out).unwrap();
 let catalog=world::WorldCatalog::original(&rom).unwrap();let mut observations=Vec::new();
 let mut display=presentation::Display::default();let mut memory=presentation::VideoMemory::blank();memory.nametables[..960].fill(0x24);memory.nametables[1024..1984].fill(0x24);
 let chr=formats::prepare_chr(&rom,&formats::original_chr_list(&rom,2).unwrap(),0).unwrap();display.apply_chr(&chr);for w in display.writes.drain(..){memory.write(w);}
 for (bank,set) in catalog.terrain.iter().enumerate(){
  for page in 0..2u16 {for y in (0..256u16).step_by(16){for x in (0..256u16).step_by(16){
   let mut cache=world::WorldCache::default();cache.attributes.fill(0xa5);cache.collision.fill(0x5a);cache.changed_this_frame=255;
   let update=cache.replace(set,world::Mutation{world_x:page*256+x+8,spawn_y:(y+8)as u8,selector:(x^y)as u8,marks_changed:true});
   let writes=update_writes(&update);if bank==0 {for w in &writes{memory.write(*w);}}
   let mut q=presentation::PreparedSecondary::source_shared();let mut cursor=(x+y+page*126)as u8;q.append_changed_shared(&mut cursor,&update,&mut cache.attributes);
   observations.push(json!({"kind":"mutation","bank":bank,"page":page,"x":x,"y":y,"writes":pairs(&writes),"cursor":cursor,"changed":cache.changed_this_frame,"collision":cache.collision.to_vec(),"attributes":cache.attributes.to_vec(),"queue":q.encoded}));
  }}}
  if bank==0{capture(&memory,&out.join("mutated-nametables.ppm"));}
 }
 let layout=world::Layout{rows:1,columns:2,rooms:vec![Some(0);2]};let set=&catalog.terrain[0];
 for stage in [0,5] {for page in 0..2 {for ring in 0..64 {for suspended in [0,1] {
  let context=world::ExpansionContext{stage,room:0,room_in_row:page,ring,split:false,stage_three_control:0,stage_four_control:0};
  let block=world::expand(set,0,context);let result=world::column(&layout,context,&block,suspended);
  observations.push(json!({"kind":"column","stage":stage,"page":page,"ring":ring,"suspended":suspended,"collision":result.collision,"attribute":result.attribute,"writes":column_writes(&result).map(|w|pairs(&w))}));
 }}}}
 for direction in [world::VerticalDirection::Up,world::VerticalDirection::Down]{for ring in 0..8 {
  let blocks:Vec<_>=(0..8).map(|i|world::expand(set,i,world::ExpansionContext{stage:0,room:0,room_in_row:1,ring,split:false,stage_three_control:0,stage_four_control:0})).collect();
  let rows=world::vertical_rows(direction,ring,&blocks,[0xa5;8]);
  let (next,copy)=world::copy_vertical_staging(direction,ring,&world::VerticalStaging{upper:rows.tiles[2],lower:rows.tiles[3],attributes:rows.attributes});
  observations.push(json!({"kind":"vertical","up":direction==world::VerticalDirection::Up,"ring":ring,"tiles":rows.tiles.iter().map(|r|r.to_vec()).collect::<Vec<_>>(),"attributes":rows.attributes,"writes":pairs(&row_writes(&rows)),"next":next,"copy":pairs(&update_writes(&copy))}));
  for previous in [false,true]{let rows=world::vertical_neighbor(direction,previous,world::Camera{x:256,room:1,previous_x:0},&world::Layout{rows:1,columns:3,rooms:vec![Some(0);3]},ring,&blocks[..4]);observations.push(json!({"kind":"neighbor","up":direction==world::VerticalDirection::Up,"ring":ring,"previous":previous,"writes":rows.as_ref().map(|r|pairs(&row_writes(r)))}));}
 }}
 for selector in [0,6]{let burst=world::changed_burst(set,selector);observations.push(json!({"kind":"burst","selector":selector,"writes":pairs(&burst_writes(&burst)),"origin":burst.particle_origin,"particles":burst.particles}));}
 fs::write(out.join("observations.json"),serde_json::to_vec(&observations).unwrap()).unwrap();println!("{} terrain cases",observations.len());
}
