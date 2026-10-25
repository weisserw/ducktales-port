//! Resource-position/read provenance required by canonical channel/debug/reset
//! aliases. These operations consume already compiled operands, never fetch or
//! classify stream bytes. Primary reads select a seed; secondary reads do not.
use super::{
    Catalog, Driver, SeedProducer, Voice,
    program::{BranchTarget, Instruction, PrimaryAction, SecondaryAction},
};
impl Driver {
    pub(super) fn primary_instruction(
        &mut self,
        catalog: &Catalog,
        voice: Voice,
    ) -> Instruction<PrimaryAction> {
        let position = self.channels[voice.index()].primary.cursor;
        let instruction = catalog.programs.primary.get(&position).copied();
        self.primary_operand(voice, instruction)
    }
    pub(super) fn secondary_instruction(
        &mut self,
        catalog: &Catalog,
    ) -> Instruction<SecondaryAction> {
        let instruction = catalog
            .programs
            .secondary
            .get(&self.secondary_cursor)
            .copied();
        self.secondary_operand(instruction)
    }
    pub(super) fn primary_operand<T>(&mut self, voice: Voice, operand: Option<T>) -> T {
        let value = operand.expect("validated primary program operand");
        let position = self.channels[voice.index()].primary.cursor;
        self.select_seed(position as u8, SeedProducer::PrimaryCursor);
        self.channels[voice.index()].primary.cursor = position.wrapping_add(1);
        value
    }
    pub(super) fn secondary_operand<T>(&mut self, operand: Option<T>) -> T {
        let value = operand.expect("validated secondary program operand");
        self.secondary_cursor = self.secondary_cursor.wrapping_add(1);
        value
    }
    pub(super) fn primary_target(&mut self, voice: Voice, target: BranchTarget) -> u16 {
        match target {
            BranchTarget::Known(destination) => {
                self.primary_operand(voice, Some(()));
                self.primary_operand(voice, Some(()));
                destination
            }
            BranchTarget::MissingLow => panic!("missing primary loop target low"),
            BranchTarget::MissingHigh => {
                self.primary_operand(voice, Some(()));
                panic!("missing primary loop target high")
            }
        }
    }
    pub(super) fn secondary_target(&mut self, target: BranchTarget) -> u16 {
        match target {
            BranchTarget::Known(destination) => {
                self.secondary_cursor = self.secondary_cursor.wrapping_add(2);
                destination
            }
            BranchTarget::MissingLow => panic!("missing secondary loop target low"),
            BranchTarget::MissingHigh => {
                self.secondary_cursor = self.secondary_cursor.wrapping_add(1);
                panic!("missing secondary loop target high")
            }
        }
    }
}
