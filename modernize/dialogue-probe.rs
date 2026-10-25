//! Bounded renderer probe: all original messages through the real text executor,
//! canonical panel commit and shared tile sampler. This is a direct-owner fixture,
//! not a natural game route or a replacement scheduler implementation.
use ducktales_port::{assets::formats, config, dialogue, presentation};
use romatlas_port::nes::ppu;
use serde_json::json;
use std::{fs, path::Path};
fn commit(display: &mut presentation::Display, memory: &mut presentation::VideoMemory) -> Vec<(u16, u8)> {
    display.commit_primary(presentation::Stride::Across);
    let writes: Vec<_> = display.writes.drain(..).map(|w| (w.address, w.value)).collect();
    for &(address, value) in &writes { memory.write(presentation::VideoWrite { address, value }); }
    writes
}
fn capture(memory: &presentation::VideoMemory, path: &Path) {
    // Common source font at CHR1000 from original list0. A simple fixed review
    // palette isolates the glyphs; this does not claim a stage palette/raster.
    let view = ppu::Background { chr: &memory.chr, chr_banks: [0,1,2,3,4,5,6,7], nametables: &memory.nametables, mirroring: ppu::Mirroring::Vertical, pattern_table: 0x1000, scroll: ppu::Scroll::default() };
    let mut bytes = b"P6\n256 64\n255\n".to_vec();
    let colors = [0x0f, 0x30, 0x10, 0x20];
    for y in 0..64 { for x in 0..256 {
        let pixel = view.pixel(x,y).unwrap();
        bytes.extend_from_slice(&ppu::color(&config::NES_RGB, colors[usize::from(pixel & 3)], 0)[..3]);
    }}
    fs::write(path, bytes).unwrap();
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let rom = fs::read(&args[1]).unwrap();
    let out = Path::new(&args[2]);fs::create_dir_all(out).unwrap();
    let catalog = dialogue::Catalog::original(&rom).unwrap();
    let font = formats::prepare_chr(&rom, &formats::original_chr_list(&rom,0).unwrap(),0).unwrap();
    let mut observations = Vec::new();
    for selector in 0..dialogue::MESSAGE_COUNT {
        let mut d = dialogue::Dialogue { message: selector as u8, ..Default::default() };
        let mut cursor = 0;
        let mut display = presentation::Display::default();
        let mut memory = presentation::VideoMemory::blank();
        memory.nametables[..960].fill(dialogue::BLANK);
        display.apply_chr(&font);
        for write in display.writes.drain(..) { memory.write(write); }
        let mut complete = false;
        let mut page = 0;
        for call in 0..256 {
            let step = dialogue::render_text(&mut d,&mut cursor,&mut display.primary,&catalog).unwrap();
            let prepared = display.primary.encoded.clone();
            let pending = display.primary.pending;
            let writes = commit(&mut display,&mut memory);
            observations.push(json!({"selector":selector,"call":call,"cursor":cursor,"line":d.line,"glyphs":d.row_glyphs,"boundary":format!("{:?}",step.boundary),"sound":step.sound,"wait":step.wait_ticks,"prepared":prepared,"pending":pending,"writes":writes,"nametables":memory.nametables}));
            match step.boundary {
                dialogue::TextBoundary::GlyphWait => (),
                dialogue::TextBoundary::MessageComplete => {
                    capture(&memory, &out.join(format!("message-{selector:02x}-page-{page}.ppm")));
                    complete = true;break;
                },
                dialogue::TextBoundary::PageComplete => {
                    capture(&memory, &out.join(format!("message-{selector:02x}-page-{page}.ppm")));
                    page += 1;
                    dialogue::scroll_rows(&mut d,&mut display.primary);
                    let prepared = display.primary.encoded.clone();
                    let writes = commit(&mut display,&mut memory);
                    observations.push(json!({"selector":selector,"scroll":page,"cursor":cursor,"prepared":prepared,"writes":writes,"nametables":memory.nametables}));
                    d.phase = dialogue::Phase::Render;d.line = 2;d.row_glyphs = 0;
                }
            }
        }
        assert!(complete,"bounded original message execution");
    }
    fs::write(out.join("observations.json"),serde_json::to_vec(&observations).unwrap()).unwrap();
    println!("Compared renderer inputs/outputs for {} events across all18 original messages",observations.len());
}
