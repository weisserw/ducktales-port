//! Bounded original status stream/message recipes. Runtime inserts remain named
//! status/progression values, never mapped RAM or retained register state.
use super::*;
pub(super) fn decode_original(rom: &[u8]) -> Result<StatusCatalog> {
    const BANK_BIAS: usize = 0xc010;
    const STREAM_LOW: usize = 0xb622;
    const STREAM_HIGH: usize = 0xb62c;
    const STREAM_COUNT: usize = 10;
    const MESSAGE_OFFSETS: usize = 0xb86c;
    const MESSAGE_RECORDS: usize = 0xb871;
    const MESSAGE_COUNT: usize = 5;
    const PAYLOAD_LOW: usize = 0xb9c1;
    const PAYLOAD_HIGH: usize = 0xb9d8;
    const PAYLOAD_COUNT: usize = 23;
    const REQUIRED_END: usize = 0x18010;
    ensure!(rom.len() >= REQUIRED_END, "truncated presentation bank");
    let pointer = |low: usize, high: usize, selector: usize| -> usize {
        BANK_BIAS
            + usize::from(rom[BANK_BIAS + low + selector])
            + (usize::from(rom[BANK_BIAS + high + selector]) << 8)
    };
    let literal = |selector: usize| -> Result<Vec<u8>> {
        let start = pointer(STREAM_LOW, STREAM_HIGH, selector);
        let count = usize::from(*rom.get(start).context("status fragment count")?) + 1;
        Ok(rom
            .get(start + 1..start + count + 1)
            .context("status fragment bounds")?
            .to_vec())
    };
    let mut fragments = vec![Fragment::Literal(Vec::new()); STREAM_COUNT];
    fragments[2] = Fragment::Reward;
    fragments[3] = Fragment::Total;
    for selector in [5, 6, 7, 8] {
        fragments[selector] = Fragment::Literal(literal(selector)?);
    }
    let absent = literal(8)?;
    for (selector, progression, mask) in [(5, 1, 2), (6, 1, 4), (7, 0, 0xff)] {
        fragments[selector] = Fragment::Conditional {
            progression,
            mask,
            present: literal(selector)?,
            absent: absent.clone(),
        };
    }
    let mut streams = vec![Vec::new(); STREAM_COUNT];
    for selector in [0, 1, 4, 9] {
        let start = pointer(STREAM_LOW, STREAM_HIGH, selector);
        streams[selector] = StatusCatalog::decode_stream(
            rom.get(start..REQUIRED_END)
                .context("status stream pointer")?,
            &fragments,
        )?;
    }
    let mut messages = Vec::new();
    for selector in 0..MESSAGE_COUNT {
        let mut cursor =
            BANK_BIAS + MESSAGE_RECORDS + usize::from(rom[BANK_BIAS + MESSAGE_OFFSETS + selector]);
        let mut records = Vec::new();
        while rom[cursor] & 0x80 == 0 {
            let destination = u16::from(rom[cursor]) << 8 | u16::from(rom[cursor + 1]);
            records.push((destination, usize::from(rom[cursor + 2])));
            cursor += 3;
        }
        messages.push(records);
    }
    let mut payload_counts = Vec::new();
    let mut payloads = Vec::new();
    for selector in 0..PAYLOAD_COUNT {
        if matches!(selector, 10 | 11) {
            payload_counts.push(None);
            payloads.push(None);
            continue;
        }
        let start = pointer(PAYLOAD_LOW, PAYLOAD_HIGH, selector);
        let count = *rom.get(start).context("message payload count")?;
        payload_counts.push(Some(count));
        payloads.push(Some(
            rom.get(start + 1..start + 2 + usize::from(count))
                .context("message payload bounds")?
                .to_vec(),
        ));
    }
    Ok(StatusCatalog {
        streams,
        messages,
        payload_counts,
        payloads,
    })
}
