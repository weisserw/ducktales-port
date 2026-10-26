//! Shared game identity and frontend configuration. Both backends use these values.
pub const ROM_SHA256: &str = "97d472f7b2c1088448d1644155b2650a546bad887b3cba571e16d7986824c169";
pub const SOURCE_SIZE: (u32, u32) = (256, 240);
pub const SOURCE_DISPLAY_ASPECT: (u32, u32) = (4, 3);
pub const DISPLAY_ASPECT: (u32, u32) = (16, 9);
/// Preserve the source display's pixel proportions with the nearest whole-pixel
/// viewport width: 256 * (16/9) / (4/3) = 341 1/3 pixels, rounded to 341.
pub const SIZE: (u32, u32) = (
    (SOURCE_SIZE.0 * DISPLAY_ASPECT.0 * SOURCE_DISPLAY_ASPECT.1
        + DISPLAY_ASPECT.1 * SOURCE_DISPLAY_ASPECT.0 / 2)
        / (DISPLAY_ASPECT.1 * SOURCE_DISPLAY_ASPECT.0),
    SOURCE_SIZE.1,
);
/// Odd additional widths leave the extra pixel on the right.
pub const SIDE_PIXELS: i16 = ((SIZE.0 - SOURCE_SIZE.0) / 2) as i16;
pub const RATE: romatlas_port::timing::TickRate = romatlas_port::timing::TickRate::NES_NTSC;
pub fn settings() -> romatlas_port::settings::Settings {
    romatlas_port::settings::Settings {
        width: 960,
        height: 540,
        ..romatlas_port::nes::default_settings()
    }
}

/// Conventional independent 2C02 RGB display palette. These hardware color
/// approximations are native renderer configuration, not cartridge graphics.
pub const NES_RGB: [[u8; 4]; 64] = {
    const RGB: [u32; 64] = [
        0x545454, 0x001e74, 0x081090, 0x300088, 0x440064, 0x5c0030, 0x540400, 0x3c1800, 0x202a00,
        0x083a00, 0x004000, 0x003c00, 0x00323c, 0x000000, 0x000000, 0x000000, 0x989698, 0x084cc4,
        0x3032ec, 0x5c1ee4, 0x8814b0, 0xa01464, 0x982220, 0x783c00, 0x545a00, 0x287200, 0x087c00,
        0x007628, 0x006678, 0x000000, 0x000000, 0x000000, 0xeceeec, 0x4c9aec, 0x787cec, 0xb062ec,
        0xe454ec, 0xec58b4, 0xec6a64, 0xd48820, 0xa0aa00, 0x74c400, 0x4cd020, 0x38cc6c, 0x38b4cc,
        0x3c3c3c, 0x000000, 0x000000, 0xeceeec, 0xa8ccec, 0xbcbcec, 0xd4b2ec, 0xecaeec, 0xecaed4,
        0xecb4b0, 0xe4c490, 0xccd278, 0xb4de78, 0xa8e290, 0x98e2b4, 0xa0d6e4, 0xa0a2a0, 0x000000,
        0x000000,
    ];
    let mut colors = [[0; 4]; 64];
    let mut index = 0;
    while index < 64 {
        colors[index] = [
            (RGB[index] >> 16) as u8,
            (RGB[index] >> 8) as u8,
            RGB[index] as u8,
            255,
        ];
        index += 1;
    }
    colors
};

/// Read-only semantic event history. Consumers use sequence cursors and detect
/// retention loss; observing never drains this bounded journal.
pub const SEMANTIC_EVENT_CAPACITY: usize = 512;
