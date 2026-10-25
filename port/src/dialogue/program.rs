//! Text actions decoded once at the asset boundary. Program positions remain
//! one-to-one with source bytes because other owners observe the shared cursor.
use super::{BLANK, MESSAGE_END, ROW_END};
use anyhow::{Context, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextInstruction {
    Glyph(u8),
    /// Blanks advance without pacing or the visible-glyph row-limit check.
    Blank,
    NextRow,
    /// Emits the final period without pacing; does not consume the cursor.
    Finish,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageProgram {
    instructions: Box<[TextInstruction]>,
}
impl MessageProgram {
    /// Decode through the first terminator, leaving adjacent source data out of
    /// the program. Only the three established control codes are special.
    pub fn from_source(encoded: &[u8]) -> Result<Self> {
        let end = encoded
            .iter()
            .position(|&code| code == MESSAGE_END)
            .context("unterminated dialogue message")?;
        let instructions = encoded[..=end]
            .iter()
            .map(|&code| match code {
                BLANK => TextInstruction::Blank,
                ROW_END => TextInstruction::NextRow,
                MESSAGE_END => TextInstruction::Finish,
                tile => TextInstruction::Glyph(tile),
            })
            .collect();
        Ok(Self { instructions })
    }
    pub fn instruction(&self, position: u8) -> Option<TextInstruction> {
        self.instructions.get(usize::from(position)).copied()
    }
    pub fn instructions(&self) -> &[TextInstruction] {
        &self.instructions
    }
    /// Source metadata tests may project codes, but gameplay never interprets
    /// or retains this encoded form beside its native instructions.
    #[cfg(test)]
    pub(super) fn source_codes(&self) -> Vec<u8> {
        self.instructions
            .iter()
            .map(|instruction| match *instruction {
                TextInstruction::Glyph(tile) => tile,
                TextInstruction::Blank => BLANK,
                TextInstruction::NextRow => ROW_END,
                TextInstruction::Finish => MESSAGE_END,
            })
            .collect()
    }
}
