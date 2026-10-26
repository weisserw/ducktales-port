//! Source-owned command ring, music/effect interpretation and register schedules.
//! All stage3 behavior entries are deliberately pending; see trace.md.
mod program;
mod source_boundary;
use program::{DurationClass, Note, PrimaryAction, SecondaryAction};
use romatlas_port::nes::apu::Apu;
pub const QUEUE_SLOTS: usize = 8;
pub const EMPTY_COMMAND: u8 = 0x88;
pub const CHANNELS: usize = 4;
pub const HEADER_COUNT: usize = 56;
pub const PRIMARY_HANDLERS: [u16; 10] = [
    0x87cf, 0x87d9, 0x87e3, 0x87f3, 0x880d, 0x8855, 0x8872, 0x8885, 0x88bf, 0x890f,
];
pub const SECONDARY_HANDLERS: [u16; 7] = [0x85d1, 0x85d9, 0x85e3, 0x85f3, 0x860d, 0x863e, 0x8654];
/// These are asset source addresses, never native CPU execution addresses.
pub const BANK_PHYSICAL_START: usize = 0xc010;
pub const BANK_CPU_START: u16 = 0x8000;
pub const HEADER_WORDS_PHYSICAL: usize = 0xca50;
pub const DURATION_WORDS_PHYSICAL: usize = 0xc97b;
pub const MIXED_PAYLOAD_PHYSICAL: std::ops::Range<usize> = 0xcac0..0xf110;
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ProducerComparison {
    BelowEmpty,
    Empty,
    AboveEmpty,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Submission {
    pub accepted: bool,
    pub comparison: ProducerComparison,
}
impl Submission {
    /// The immediate caller's random addition input, not global processor flags.
    pub fn first_actor_increment(self) -> crate::progression::FirstActorIncrement {
        if self.random_increment() {
            crate::progression::FirstActorIncrement::One
        } else {
            crate::progression::FirstActorIncrement::Zero
        }
    }
    pub fn random_increment(self) -> bool {
        self.comparison != ProducerComparison::BelowEmpty
    }
}
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CommandQueue {
    #[serde(with = "crate::snapshot::array")]
    pub slots: [u8; QUEUE_SLOTS],
    pub producer: usize,
    pub consumer: usize,
    pub stage_command: u8,
    #[serde(skip)]
    events: Option<crate::events::Events>,
}
// Equality describes source ring state, not an optional shared observer handle.
impl PartialEq for CommandQueue {
    fn eq(&self, other: &Self) -> bool {
        self.slots == other.slots
            && self.producer == other.producer
            && self.consumer == other.consumer
            && self.stage_command == other.stage_command
    }
}
impl Eq for CommandQueue {}
impl Default for CommandQueue {
    fn default() -> Self {
        Self {
            slots: [EMPTY_COMMAND; QUEUE_SLOTS],
            producer: 0,
            consumer: 0,
            stage_command: 0,
            events: None,
        }
    }
}
impl CommandQueue {
    pub(crate) fn observe_with(&mut self, events: crate::events::Events) {
        self.events = Some(events);
    }
    pub fn submit(&mut self, command: u8) -> Submission {
        self.submit_request(command, false)
    }
    fn submit_request(&mut self, command: u8, stage_command: bool) -> Submission {
        let producer = self.producer;
        let previous = self.slots[self.producer];
        let comparison = match previous.cmp(&EMPTY_COMMAND) {
            std::cmp::Ordering::Less => ProducerComparison::BelowEmpty,
            std::cmp::Ordering::Equal => ProducerComparison::Empty,
            std::cmp::Ordering::Greater => ProducerComparison::AboveEmpty,
        };
        let accepted = previous == EMPTY_COMMAND;
        if accepted {
            self.slots[self.producer] = command;
            self.producer = (self.producer + 1) & 7;
        }
        if let Some(events) = &self.events {
            events.record("AudioCommand", serde_json::json!({"command":command,"stage_command":stage_command,"producer":producer,"previous":previous,"accepted":accepted,"comparison":format!("{comparison:?}"),"next_producer":self.producer}));
        }
        Submission {
            accepted,
            comparison,
        }
    }
    pub fn submit_stage(&mut self, command: u8) -> Submission {
        self.stage_command = command;
        self.submit_request(command, true)
    }
    pub fn drain(&mut self) -> Vec<u8> {
        let mut commands = Vec::new();
        while self.slots[self.consumer] != EMPTY_COMMAND {
            let command = self.slots[self.consumer];
            commands.push(command);
            if let Some(events) = &self.events {
                events.record(
                    "AudioDequeued",
                    serde_json::json!({"command":command,"consumer":self.consumer}),
                );
            }
            self.slots[self.consumer] = EMPTY_COMMAND;
            self.consumer = (self.consumer + 1) & 7;
        }
        commands
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Voice {
    PulseOne,
    PulseTwo,
    Triangle,
    Noise,
}
impl Voice {
    fn index(self) -> usize {
        match self {
            Self::PulseOne => 0,
            Self::PulseTwo => 1,
            Self::Triangle => 2,
            Self::Noise => 3,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Header {
    Primary {
        priority: u8,
        cursors: [u16; CHANNELS],
        preset_base: u16,
    },
    Secondary {
        priority: u8,
        mask: u8,
        cursor: u16,
    },
}
/// Fragment identity belongs to B02; original encoded addresses are decoded only
/// in this immutable catalog/loader boundary. Synthetic fragments are authored.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fragment {
    pub origin: u16,
    pub bytes: Vec<u8>,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Catalog {
    pub headers: Vec<Header>,
    fragments: Vec<Fragment>,
    programs: program::Programs,
    pub duration: [u8; 8],
    pub alternate_duration: [u8; 8],
}
impl Catalog {
    /// Synthetic/file asset construction compiles both stream roles once.
    pub fn from_fragments(fragments: Vec<Fragment>) -> Self {
        let programs = program::Programs::compile(&fragments);
        Self {
            fragments,
            programs,
            ..Self::default()
        }
    }
    pub fn add_fragment(&mut self, fragment: Fragment) {
        self.fragments.push(fragment);
        self.programs = program::Programs::compile(&self.fragments);
    }
    /// File-authored assets may override a source data/stream location. This is
    /// an import operation, so both compiled roles are rebuilt before playback.
    pub fn prepend_fragment(&mut self, fragment: Fragment) {
        self.fragments.insert(0, fragment);
        self.programs = program::Programs::compile(&self.fragments);
    }

    /// Only independently bounded audio records enter this catalog. The mixed
    /// source record remains immutable and its trailing bitmap data stays outside
    /// the audio consumer domain.
    pub fn from_assets(assets: &crate::assets::Assets) -> anyhow::Result<Self> {
        let mut image = vec![0; 0xe903];
        for id in [
            "AudioDurationAndPeriodData_0C97BBytes",
            "AudioCommandHeaderPointers_0CA50Bytes",
            "AudioStreamsAndUnresolvedBitmapData_0CAC0Bytes",
        ] {
            let recipe = crate::recipes::original_assets()
                .into_iter()
                .find(|r| r.id == id)
                .ok_or_else(|| anyhow::anyhow!("missing audio recipe {id}"))?;
            let asset = assets
                .source
                .get(id)
                .ok_or_else(|| anyhow::anyhow!("missing audio record {id}"))?;
            anyhow::ensure!(
                asset.bytes.len() == recipe.length,
                "invalid audio record length"
            );
            let end = (recipe.offset + recipe.length).min(image.len());
            image[recipe.offset..end].copy_from_slice(&asset.bytes[..end - recipe.offset]);
        }
        Self::from_rom(&image)
    }
    pub fn from_rom(rom: &[u8]) -> anyhow::Result<Self> {
        anyhow::ensure!(rom.len() >= 0xe903, "truncated audio source");
        let mut catalog = Self::from_fragments(vec![Fragment {
            origin: 0x896b,
            bytes: rom[0xc97b..0xe903].to_vec(),
        }]);
        catalog.duration = rom[0xc97b..0xc983].try_into()?;
        catalog.alternate_duration = rom[0xc983..0xc98b].try_into()?;
        for index in 0..HEADER_COUNT {
            let at = HEADER_WORDS_PHYSICAL + index * 2;
            let pointer = u16::from_le_bytes([rom[at], rom[at + 1]]);
            let priority = catalog.read(pointer)?;
            let word = |offset: u16| -> anyhow::Result<u16> {
                Ok(u16::from_le_bytes([
                    catalog.read(pointer.wrapping_add(offset))?,
                    catalog.read(pointer.wrapping_add(offset + 1))?,
                ]))
            };
            let header = if priority & 15 != 0 {
                let mut cursors = [0; CHANNELS];
                for (channel, cursor) in cursors.iter_mut().enumerate() {
                    *cursor = word(1 + channel as u16 * 2)?;
                }
                Header::Primary {
                    priority,
                    cursors,
                    preset_base: word(9)?,
                }
            } else {
                Header::Secondary {
                    priority,
                    mask: catalog.read(pointer.wrapping_add(1))? & 15,
                    cursor: pointer.wrapping_add(2),
                }
            };
            catalog.headers.push(header);
        }
        Ok(catalog)
    }
    pub fn primary_targets(&self) -> [u16; 10] {
        PRIMARY_HANDLERS
    }
    pub fn secondary_targets(&self) -> [u16; 7] {
        SECONDARY_HANDLERS
    }
    pub fn read(&self, address: u16) -> anyhow::Result<u8> {
        program::source_byte(&self.fragments, address)
            .ok_or_else(|| anyhow::anyhow!("audio data address {address:04X} outside catalog"))
    }
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Envelope {
    pub interval: u8,
    pub counter: u8,
    pub step_level: u8,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Modulation {
    pub pitch_interval: u8,
    pub pitch_shape: u8,
    pub level_interval: u8,
    pub level_step: i8,
    pub pitch_counter: u8,
    pub pitch_phase: u8,
    pub pitch_offset: u16,
    pub cached_high: u8,
    pub level_counter: u8,
    pub level: u8,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Primary {
    pub cursor: u16,
    pub duration: u16,
    pub scale: u8,
    /// Low seven repeat bits and continuation bit share an original encoded byte.
    pub repeat_control: u8,
    /// High three deferred-note bits and low five preset bits share this encoding.
    pub note_control: u8,
    pub period_table: u16,
    pub pitch_step: i8,
    pub period: u16,
    pub control: u8,
    pub envelope: Envelope,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Secondary {
    pub pitch_step: i8,
    pub period: u16,
    pub control: u8,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Channel {
    pub primary: Primary,
    pub secondary: Secondary,
    pub modulation: Modulation,
}
impl Channel {
    fn encoded_byte(&self, offset: usize) -> u8 {
        let p = self.primary;
        let s = self.secondary;
        let m = self.modulation;
        match offset {
            0 => p.cursor as u8,
            1 => (p.cursor >> 8) as u8,
            2 => p.duration as u8,
            3 => (p.duration >> 8) as u8,
            4 => p.scale,
            5 => p.repeat_control,
            6 => p.note_control,
            7 => p.period_table as u8,
            8 => (p.period_table >> 8) as u8,
            9 => p.pitch_step as u8,
            10 => p.period as u8,
            11 => (p.period >> 8) as u8,
            12 => p.control,
            13 => p.envelope.interval,
            14 => p.envelope.counter,
            15 => p.envelope.step_level,
            16 => s.pitch_step as u8,
            17 => s.period as u8,
            18 => (s.period >> 8) as u8,
            19 => s.control,
            20 => m.pitch_interval,
            21 => m.pitch_shape,
            22 => m.level_interval,
            23 => m.level_step as u8,
            24 => m.pitch_counter,
            25 => m.pitch_phase,
            26 => m.pitch_offset as u8,
            27 => (m.pitch_offset >> 8) as u8,
            28 => m.cached_high,
            29 => m.level_counter,
            30 => m.level,
            _ => unreachable!(),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegisterWrite {
    pub address: u16,
    pub value: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SeedProducer {
    ResetZero,
    PrimaryCursor,
    InlineTarget,
    ControlParameter,
    LoopDestination,
    ModulationEnd,
    PrimaryControl,
    SecondaryControl,
    EnvelopeEndpoint,
    EnvelopeInterval,
    DeferredNote,
    PeriodTable,
    PresetIndex,
    PresetPointer,
    ComputedLevel,
    PitchShape,
    NoiseMode,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SeedSelector {
    pub selector: u8,
    pub producer: SeedProducer,
}
impl Default for SeedSelector {
    fn default() -> Self {
        Self {
            selector: 0,
            producer: SeedProducer::ResetZero,
        }
    }
}
#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Driver {
    pub reset_selector: SeedSelector,
    #[serde(with = "crate::snapshot::array")]
    pub channels: [Channel; CHANNELS],
    pub priority: u8,
    pub secondary_mask: u8,
    pub secondary_cursor: u16,
    pub secondary_repeat: u8,
    pub shared_delay: u8,
    pub tempo: u8,
    pub fade: u8,
    pub fade_level: u8,
    pub phase: u8,
    pub busy: bool,
    pub preset_base: u16,
    pub frames: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Install {
    Rejected,
    Primary,
    Secondary,
    Tempo,
    Fade,
    StopSecondary,
    StopPrimary,
}
/// A concrete caller-derived control/level byte, not a raw record index.
/// No Default: secondary lowF must resolve its actual predecessor producer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResetLevel {
    pub value: u8,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reset {
    InvalidatePeriod,
    KeepPeriod,
}
impl Driver {
    fn silence(voice: Voice, writes: &mut Vec<RegisterWrite>) {
        let base = 0x4000 + voice.index() as u16 * 4;
        writes.push(RegisterWrite {
            address: base,
            value: if voice == Voice::Triangle { 0 } else { 0x30 },
        });
        if voice == Voice::Triangle {
            writes.push(RegisterWrite {
                address: base + 2,
                value: 0,
            });
            writes.push(RegisterWrite {
                address: base + 3,
                value: 0,
            });
        }
    }
    fn resume_primary(
        &mut self,
        catalog: &Catalog,
        voice: Voice,
        writes: &mut Vec<RegisterWrite>,
    ) -> u8 {
        let channel = &mut self.channels[voice.index()];
        if channel.primary.period != 0 {
            channel.modulation.cached_high = 0xff;
        } else {
            Self::silence(voice, writes);
            if channel.primary.cursor == 0 {
                return 4 - voice.index() as u8;
            }
        }
        let preset = channel.primary.note_control & 31;
        // Command restoration uses81CA's indexed-copy helper, not88CB's
        // stream helper. It preserves the indexed reset producer and returns
        // the consumed preset offset plus four as the next command parameter.
        self.install_preset_fields(catalog, voice, preset);
        preset.wrapping_mul(4).wrapping_add(4)
    }
    /// The parameter is the traced command producer's value, never CPU state.
    pub fn command(
        &mut self,
        catalog: &Catalog,
        command: u8,
        parameter: u8,
        writes: &mut Vec<RegisterWrite>,
    ) -> Install {
        self.command_with_parameter(catalog, command, parameter, writes)
            .0
    }
    fn command_with_parameter(
        &mut self,
        catalog: &Catalog,
        command: u8,
        parameter: u8,
        writes: &mut Vec<RegisterWrite>,
    ) -> (Install, u8) {
        match command {
            0xfc => {
                self.tempo = parameter.wrapping_add(1);
                return (Install::Tempo, self.tempo);
            }
            0xfd => {
                self.fade = parameter;
                self.fade_level = 1;
                self.phase &= 1;
                return (Install::Fade, parameter);
            }
            0xfe => {
                self.priority &= 15;
                let mut parameter = parameter;
                for voice in [
                    Voice::PulseOne,
                    Voice::PulseTwo,
                    Voice::Triangle,
                    Voice::Noise,
                ] {
                    if self.secondary_mask & 1 != 0 {
                        let channel = &mut self.channels[voice.index()];
                        channel.secondary = Secondary::default();
                        channel.modulation = Modulation::default();
                        parameter = self.resume_primary(catalog, voice, writes);
                    }
                    self.secondary_mask = self.secondary_mask.rotate_right(1);
                }
                self.busy = false;
                return (Install::StopSecondary, parameter);
            }
            0xff => {
                self.priority &= 0xf0;
                self.tempo = 0;
                self.fade = 0;
                for channel in &mut self.channels {
                    channel.primary.cursor = 0;
                }
                return (Install::StopPrimary, parameter);
            }
            _ => {}
        }
        let Some(header) = catalog.headers.get(usize::from(command)).cloned() else {
            return (Install::Rejected, parameter);
        };
        match header {
            Header::Primary {
                priority,
                cursors,
                preset_base,
            } => {
                if priority < self.priority & 15 {
                    return (Install::Rejected, 0);
                }
                self.priority = (self.priority & 0xf0) | priority;
                self.tempo = 0;
                self.fade = 0;
                for (index, cursor) in cursors.into_iter().enumerate() {
                    self.channels[index].primary = Primary {
                        cursor,
                        ..Primary::default()
                    };
                    if self.secondary_mask & (1 << index) == 0 {
                        self.channels[index].secondary = Secondary::default();
                        self.channels[index].modulation = Modulation::default();
                    }
                }
                self.preset_base = preset_base;
                self.busy = false;
                (Install::Primary, 3)
            }
            Header::Secondary {
                priority,
                mask,
                cursor,
            } => {
                if priority < self.priority & 0xf0 {
                    return (Install::Rejected, 0);
                }
                self.priority = (self.priority & 15) | priority;
                self.secondary_cursor = cursor;
                self.shared_delay = 0;
                self.secondary_repeat = 0;
                let old = self.secondary_mask;
                self.secondary_mask = mask;
                let mut parameter = 1;
                for voice in [
                    Voice::PulseOne,
                    Voice::PulseTwo,
                    Voice::Triangle,
                    Voice::Noise,
                ] {
                    let bit = 1 << voice.index();
                    if mask & bit != 0 {
                        self.channels[voice.index()].secondary = Secondary::default();
                        self.channels[voice.index()].modulation = Modulation::default();
                        parameter = 0;
                    } else if old & bit != 0 {
                        parameter = self.resume_primary(catalog, voice, writes);
                    }
                }
                self.busy = false;
                (Install::Secondary, parameter)
            }
        }
    }
    pub fn interrupt(
        &mut self,
        queue: &mut CommandQueue,
        catalog: &Catalog,
        status_selector: u8,
    ) -> Vec<RegisterWrite> {
        self.interrupt_with_alias(queue, catalog, status_selector, &mut |address| {
            panic!("audio interrupt requires canonical display byte {address:04X}")
        })
    }
    pub fn interrupt_with_alias(
        &mut self,
        queue: &mut CommandQueue,
        catalog: &Catalog,
        status_selector: u8,
        external: &mut impl FnMut(u16) -> u8,
    ) -> Vec<RegisterWrite> {
        let mut writes = Vec::new();
        let mut parameter = 0;
        for command in queue.drain() {
            parameter = self
                .command_with_parameter(catalog, command, parameter, &mut writes)
                .1;
        }
        writes.extend(self.frame_with_alias(catalog, status_selector, external));
        writes
    }
    pub fn frame(&mut self, catalog: &Catalog, status_selector: u8) -> Vec<RegisterWrite> {
        self.frame_with_alias(catalog, status_selector, &mut |address| {
            panic!("audio frame requires canonical display byte {address:04X}")
        })
    }
    pub fn frame_with_alias(
        &mut self,
        catalog: &Catalog,
        status_selector: u8,
        external: &mut impl FnMut(u16) -> u8,
    ) -> Vec<RegisterWrite> {
        let mut writes = vec![
            RegisterWrite {
                address: 0x4001,
                value: 8,
            },
            RegisterWrite {
                address: 0x4005,
                value: 8,
            },
            RegisterWrite {
                address: 0x4015,
                value: 15,
            },
        ];
        self.phase = self.phase.wrapping_add(1);
        self.frames += 1;
        if self.busy {
            return writes;
        }
        for voice in [
            Voice::PulseOne,
            Voice::PulseTwo,
            Voice::Triangle,
            Voice::Noise,
        ] {
            let channel = &mut self.channels[voice.index()];
            channel.modulation.pitch_counter = channel.modulation.pitch_counter.wrapping_add(1);
            channel.modulation.level_counter = channel.modulation.level_counter.wrapping_add(1);
            if self.secondary_mask & 1 != 0 {
                writes.extend(self.secondary_step_with_alias(catalog, voice, external));
            }
            if status_selector & 1 == 0 && self.channels[voice.index()].primary.cursor != 0 {
                let envelope = &mut self.channels[voice.index()].primary.envelope;
                envelope.counter = envelope.counter.wrapping_add(1);
                writes.extend(self.primary_step_with_alias(catalog, voice, external));
            } else if self.secondary_mask & 1 == 0 {
                Self::silence(voice, &mut writes);
            }
            self.secondary_mask = self.secondary_mask.rotate_right(1);
        }
        self.secondary_mask >>= 4;
        if self.fade & 0x7f != 0 && self.phase == self.fade & 0x7f {
            self.phase &= 1;
            self.fade_level = self.fade_level.wrapping_add(1);
            if self.fade_level == 16 {
                self.fade_level = 15;
                if self.fade & 0x80 == 0 {
                    self.fade = 0;
                }
            }
        }
        if self.shared_delay != 0 {
            self.shared_delay = self.shared_delay.wrapping_sub(1);
        }
        writes
    }
    fn select_seed(&mut self, selector: u8, producer: SeedProducer) {
        self.reset_selector = SeedSelector { selector, producer };
    }
    fn reset_selected(&mut self, voice: Voice, reset: Reset, external: &mut impl FnMut(u16) -> u8) {
        let value = if self.channels[voice.index()].modulation.level_interval & 0x80 == 0 {
            0
        } else {
            let selector = self.reset_selector;
            let offset = voice.index() * 31 + usize::from(selector.selector);
            if offset < CHANNELS * 31 {
                self.channels[offset / 31].encoded_byte(offset % 31)
            } else if offset == 124 {
                self.preset_base as u8
            } else if offset == 125 {
                (self.preset_base >> 8) as u8
            } else {
                external(0x370 + offset as u16)
            }
        };
        self.reset_modulation(voice, ResetLevel { value }, reset);
    }
    fn initialize_envelope(&mut self, voice: Voice) {
        let envelope = &mut self.channels[voice.index()].primary.envelope;
        envelope.counter = 0;
        let endpoint = if envelope.interval & 0x80 != 0 { 15 } else { 0 };
        envelope.step_level = (envelope.step_level & 0xf0) | endpoint;
        self.select_seed(endpoint, SeedProducer::EnvelopeEndpoint);
    }
    pub fn primary_step(&mut self, catalog: &Catalog, voice: Voice) -> Vec<RegisterWrite> {
        self.primary_step_with_alias(catalog, voice, &mut |address| {
            panic!("audio reset needs borrowed canonical display byte {address:04X}")
        })
    }
    pub fn primary_step_with_alias(
        &mut self,
        catalog: &Catalog,
        voice: Voice,
        external: &mut impl FnMut(u16) -> u8,
    ) -> Vec<RegisterWrite> {
        let mut writes = Vec::new();
        for _ in 0..self.tempo.max(1) {
            if self.channels[voice.index()].primary.repeat_control & 0x80 != 0
                && self.phase & 1 != 0
            {
                self.primary_tick(catalog, voice, external, &mut writes);
            }
            self.primary_tick(catalog, voice, external, &mut writes);
        }
        writes
    }
    fn primary_tick(
        &mut self,
        catalog: &Catalog,
        voice: Voice,
        external: &mut impl FnMut(u16) -> u8,
        writes: &mut Vec<RegisterWrite>,
    ) {
        let primary = &mut self.channels[voice.index()].primary;
        if primary.duration != 0 {
            primary.duration = primary.duration.wrapping_sub(4);
            if primary.duration != 0 {
                if primary.period != 0 {
                    let level = self.level(voice);
                    if self.secondary_mask & 1 == 0 {
                        writes.push(RegisterWrite {
                            address: 0x4000 + voice.index() as u16 * 4,
                            value: level,
                        });
                    }
                    writes.extend(self.pitch(voice));
                }
                return;
            }
        }
        self.channels[voice.index()].primary.repeat_control &= 0x7f;
        self.primary_decode(catalog, voice, external, writes);
    }
    fn primary_decode(
        &mut self,
        catalog: &Catalog,
        voice: Voice,
        external: &mut impl FnMut(u16) -> u8,
        writes: &mut Vec<RegisterWrite>,
    ) {
        loop {
            let instruction = self.primary_instruction(catalog, voice);
            if let Some(seed) = instruction.dispatch_seed {
                self.select_seed(seed, SeedProducer::InlineTarget);
            }
            match instruction.action {
                PrimaryAction::Deferred(count) => {
                    self.primary_decode(catalog, voice, external, writes);
                    let primary = &mut self.channels[voice.index()].primary;
                    primary.note_control = (primary.note_control & 31) | (count << 5);
                    self.select_seed(count << 5, SeedProducer::DeferredNote);
                    return;
                }
                PrimaryAction::Continue => {
                    self.channels[voice.index()].primary.repeat_control |= 0x80;
                }
                PrimaryAction::Note(note) => {
                    self.primary_note(catalog, voice, note, external, writes);
                    return;
                }
                PrimaryAction::Scale(value) => {
                    let value = self.primary_operand(voice, value);
                    self.channels[voice.index()].primary.scale = value;
                }
                PrimaryAction::PitchStep(value) => {
                    let value = self.primary_operand(voice, value);
                    self.channels[voice.index()].primary.pitch_step = value;
                }
                PrimaryAction::ControlHigh(value) | PrimaryAction::ControlLow(value) => {
                    let high = matches!(instruction.action, PrimaryAction::ControlHigh(_));
                    let value = self.primary_operand(voice, value);
                    let control = &mut self.channels[voice.index()].primary.control;
                    *control = if high {
                        (*control & 0x3f) | value
                    } else if voice == Voice::Triangle {
                        value
                    } else {
                        (*control & 0xc0) | value
                    };
                    if high || voice != Voice::Triangle {
                        self.select_seed(value, SeedProducer::ControlParameter);
                    }
                }
                PrimaryAction::Repeat { count, target } => {
                    let count = self.primary_operand(voice, count);
                    let repeat = self.channels[voice.index()].primary.repeat_control & 0x7f;
                    if count != 0 && count == repeat {
                        let primary = &mut self.channels[voice.index()].primary;
                        primary.repeat_control &= 0x80;
                        primary.cursor = primary.cursor.wrapping_add(2);
                    } else {
                        if count != 0 {
                            let primary = &mut self.channels[voice.index()].primary;
                            primary.repeat_control =
                                (primary.repeat_control & 0x80) | repeat.wrapping_add(1);
                        }
                        let destination = self.primary_target(voice, target);
                        self.channels[voice.index()].primary.cursor = destination;
                    }
                }
                PrimaryAction::PeriodTable(location) => {
                    let location = self.primary_operand(voice, location);
                    self.channels[voice.index()].primary.period_table = location;
                    self.select_seed(0x7b, SeedProducer::PeriodTable);
                }
                PrimaryAction::AlternateNote(note) => {
                    let note = self.primary_operand(voice, note);
                    self.primary_note(catalog, voice, note, external, writes);
                    return;
                }
                PrimaryAction::Envelope { interval, packed } => {
                    let interval = self.primary_operand(voice, interval);
                    let packed = self.primary_operand(voice, packed);
                    self.channels[voice.index()].primary.envelope.interval = interval;
                    self.channels[voice.index()].primary.envelope.step_level = packed;
                    self.select_seed(interval, SeedProducer::EnvelopeInterval);
                    if interval & 0x7f != 0 {
                        self.initialize_envelope(voice);
                    }
                }
                PrimaryAction::Preset(value) => {
                    let value = self.primary_operand(voice, value);
                    let primary = &mut self.channels[voice.index()].primary;
                    primary.note_control = (primary.note_control & 0xe0) | value;
                    self.select_seed(value, SeedProducer::PresetIndex);
                    if self.secondary_mask & 1 == 0 {
                        self.load_preset(catalog, voice, value);
                    }
                }
                PrimaryAction::Stop => {
                    self.channels[voice.index()].primary.cursor = 0;
                    self.priority &= 0xf0;
                    if self.secondary_mask & 1 == 0 {
                        Self::silence(voice, writes);
                    }
                    return;
                }
                PrimaryAction::Invalid(value) => {
                    panic!("unsupported primary instruction {value:02X}")
                }
            }
        }
    }
    fn primary_note(
        &mut self,
        catalog: &Catalog,
        voice: Voice,
        note: Note,
        external: &mut impl FnMut(u16) -> u8,
        writes: &mut Vec<RegisterWrite>,
    ) {
        let duration = match note.duration {
            DurationClass::Standard(class) => catalog.duration[usize::from(class)],
            DurationClass::Alternate(class) => catalog.alternate_duration[usize::from(class)],
        };
        let primary = &mut self.channels[voice.index()].primary;
        let scale = if primary.scale == 0 {
            256
        } else {
            u16::from(primary.scale)
        };
        primary.duration = u16::from(duration).wrapping_mul(scale);
        if primary.note_control & 0xe0 != 0 {
            let deferred = (primary.note_control & 0xe0).wrapping_sub(0x20);
            primary.note_control = (primary.note_control & 31) | deferred;
            self.select_seed(deferred, SeedProducer::DeferredNote);
            if self.secondary_mask & 1 == 0 {
                self.reset_selected(voice, Reset::KeepPeriod, external);
            }
            return;
        }
        let key = note.key;
        primary.period = if key == 0 {
            0
        } else if voice == Voice::Noise {
            u16::from(key)
        } else {
            let address = primary.period_table.wrapping_add(u16::from(key) * 2);
            u16::from_le_bytes([
                catalog.read(address).expect("audio period low"),
                catalog
                    .read(address.wrapping_add(1))
                    .expect("audio period high"),
            ])
        };
        let interval = primary.envelope.interval;
        self.select_seed(interval, SeedProducer::EnvelopeInterval);
        if interval & 0x7f != 0 {
            self.initialize_envelope(voice);
        }
        if self.secondary_mask & 1 == 0 {
            self.select_seed(12, SeedProducer::PrimaryControl);
            self.reset_selected(voice, Reset::InvalidatePeriod, external);
            Self::silence(voice, writes);
        }
    }
    pub fn secondary_step(&mut self, catalog: &Catalog, voice: Voice) -> Vec<RegisterWrite> {
        self.secondary_step_with_alias(catalog, voice, &mut |address| {
            panic!("audio reset needs borrowed canonical display byte {address:04X}")
        })
    }
    pub fn secondary_step_with_alias(
        &mut self,
        catalog: &Catalog,
        voice: Voice,
        external: &mut impl FnMut(u16) -> u8,
    ) -> Vec<RegisterWrite> {
        let mut writes = Vec::new();
        if self.shared_delay != 0 {
            if self.channels[voice.index()].secondary.period != 0 {
                let level = self.level_stream(voice, true);
                writes.push(RegisterWrite {
                    address: 0x4000 + voice.index() as u16 * 4,
                    value: level,
                });
                writes.extend(self.pitch_stream(voice, true));
            }
            return writes;
        }
        loop {
            let instruction = self.secondary_instruction(catalog);
            if let Some(seed) = instruction.dispatch_seed {
                self.select_seed(seed, SeedProducer::InlineTarget);
            }
            match instruction.action {
                SecondaryAction::KeepPeriod(parameter) => {
                    self.secondary_operand(parameter);
                    self.reset_selected(voice, Reset::KeepPeriod, external);
                    return writes;
                }
                SecondaryAction::Period(period) => {
                    let period = self.secondary_operand(period);
                    self.channels[voice.index()].secondary.period = period;
                    self.select_seed(19, SeedProducer::SecondaryControl);
                    self.reset_selected(voice, Reset::InvalidatePeriod, external);
                    Self::silence(voice, &mut writes);
                    return writes;
                }
                SecondaryAction::Delay(value) => {
                    self.shared_delay = self.secondary_operand(value);
                }
                SecondaryAction::PitchStep(value) => {
                    let value = self.secondary_operand(value);
                    self.channels[voice.index()].secondary.pitch_step = value;
                }
                SecondaryAction::ControlHigh(value) | SecondaryAction::ControlLow(value) => {
                    let high = matches!(instruction.action, SecondaryAction::ControlHigh(_));
                    let value = self.secondary_operand(value);
                    let control = &mut self.channels[voice.index()].secondary.control;
                    *control = if high {
                        (*control & 0x3f) | value
                    } else if voice == Voice::Triangle {
                        value
                    } else {
                        (*control & 0xc0) | value
                    };
                    if high || voice != Voice::Triangle {
                        self.select_seed(value, SeedProducer::ControlParameter);
                    }
                }
                SecondaryAction::Repeat { count, target } => {
                    let count = self.secondary_operand(count);
                    if count != 0 && count == self.secondary_repeat {
                        self.secondary_repeat = 0;
                        self.secondary_cursor = self.secondary_cursor.wrapping_add(2);
                    } else {
                        if count != 0 {
                            self.secondary_repeat = self.secondary_repeat.wrapping_add(1);
                        }
                        let destination = self.secondary_target(target);
                        self.select_seed(destination as u8, SeedProducer::LoopDestination);
                        self.secondary_cursor = destination;
                    }
                }
                SecondaryAction::Modulation {
                    pitch_interval,
                    pitch_shape,
                    level_interval,
                    level_step,
                } => {
                    let pitch_interval = self.secondary_operand(pitch_interval);
                    let pitch_shape = self.secondary_operand(pitch_shape);
                    let level_interval = self.secondary_operand(level_interval);
                    let level_step = self.secondary_operand(level_step);
                    let modulation = &mut self.channels[voice.index()].modulation;
                    modulation.pitch_interval = pitch_interval;
                    modulation.pitch_shape = pitch_shape;
                    modulation.level_interval = level_interval;
                    modulation.level_step = level_step;
                    self.select_seed(24, SeedProducer::ModulationEnd);
                }
                SecondaryAction::End => {
                    self.secondary_cursor = self.secondary_cursor.wrapping_sub(1);
                    self.priority &= 15;
                    self.secondary_mask &= 0xfe;
                    let primary = self.channels[voice.index()].primary;
                    if primary.period == 0 {
                        Self::silence(voice, &mut writes);
                        if primary.cursor == 0 {
                            return writes;
                        }
                    }
                    self.load_preset(catalog, voice, primary.note_control & 31);
                    self.select_seed(12, SeedProducer::PrimaryControl);
                    self.reset_selected(voice, Reset::InvalidatePeriod, external);
                    return writes;
                }
                SecondaryAction::Invalid(value) => {
                    panic!("unsupported secondary instruction {value:02X}")
                }
            }
        }
    }
    pub fn level(&mut self, voice: Voice) -> u8 {
        self.level_stream(voice, false)
    }
    fn level_stream(&mut self, voice: Voice, secondary: bool) -> u8 {
        let channel = &mut self.channels[voice.index()];
        let control = if secondary {
            channel.secondary.control
        } else {
            channel.primary.control
        };
        let mut level = if voice == Voice::Triangle {
            control
        } else {
            control & 15
        };
        if !secondary {
            if self.fade & 0x7f != 0 {
                let count = if voice == Voice::Triangle {
                    self.fade_level.wrapping_mul(13)
                } else {
                    self.fade_level
                };
                if self.fade & 0x80 == 0 {
                    if count != 0 {
                        level = level.min(count);
                    }
                } else {
                    for _ in 0..if count == 0 { 256 } else { u16::from(count) } {
                        level = level.wrapping_sub(1);
                        if level == 0 {
                            break;
                        }
                    }
                }
            }
            let envelope = &mut channel.primary.envelope;
            if voice != Voice::Triangle && envelope.interval & 0x7f != 0 {
                let mut value = i16::from(envelope.step_level & 15);
                if envelope.counter == envelope.interval & 0x7f {
                    envelope.counter = 0;
                    let step = i16::from(envelope.step_level >> 4);
                    value += if envelope.interval & 0x80 != 0 {
                        -step
                    } else {
                        step
                    };
                }
                level = value.clamp(0, i16::from(level)) as u8;
                envelope.step_level = (envelope.step_level & 0xf0) | level;
            }
        }
        if !secondary && self.secondary_mask & 1 != 0 {
            self.select_seed(level, SeedProducer::ComputedLevel);
            return level;
        }
        let modulation = &mut channel.modulation;
        if modulation.level_interval & 0x7f != 0 {
            if modulation.level_counter == modulation.level_interval & 0x7f {
                modulation.level_counter = 0;
                let next = modulation.level.wrapping_add_signed(modulation.level_step);
                modulation.level = if next == 0 || next & 0x80 != 0 {
                    modulation.level_step = modulation.level_step.wrapping_neg();
                    1
                } else if next >= 16 {
                    modulation.level_step = modulation.level_step.wrapping_neg();
                    15
                } else {
                    next
                };
            }
            level = level.min(modulation.level);
        }
        let output = if voice == Voice::Triangle {
            level
        } else {
            (control & 0xf0) | level
        };
        self.select_seed(output, SeedProducer::ComputedLevel);
        output
    }
    pub fn pitch(&mut self, voice: Voice) -> Vec<RegisterWrite> {
        self.pitch_stream(voice, false)
    }
    fn pitch_stream(&mut self, voice: Voice, secondary: bool) -> Vec<RegisterWrite> {
        let channel = &mut self.channels[voice.index()];
        let period = if secondary {
            channel.secondary.period = channel
                .secondary
                .period
                .wrapping_add_signed(i16::from(channel.secondary.pitch_step));
            channel.secondary.period
        } else {
            channel.primary.period = channel
                .primary
                .period
                .wrapping_add_signed(i16::from(channel.primary.pitch_step));
            channel.primary.period
        };
        if !secondary && self.secondary_mask & 1 != 0 {
            return Vec::new();
        }
        let mut next_selector = None;
        let modulation = &mut channel.modulation;
        if modulation.pitch_interval & 0x7f != 0
            && modulation.pitch_counter == modulation.pitch_interval & 0x7f
        {
            modulation.pitch_counter = 0;
            let delta = i16::from((modulation.pitch_shape >> 5) & 7);
            modulation.pitch_offset = modulation.pitch_offset.wrapping_add_signed(
                if modulation.pitch_phase & 0x80 == 0 {
                    delta
                } else {
                    -delta
                },
            );
            next_selector = Some(SeedSelector {
                selector: modulation.pitch_shape & 31,
                producer: SeedProducer::PitchShape,
            });
            modulation.pitch_phase = modulation.pitch_phase.wrapping_add(1);
            if modulation.pitch_phase & 0x7f == modulation.pitch_shape & 0x1f {
                modulation.pitch_phase &= 0x80;
                if modulation.pitch_interval & 0x80 == 0 {
                    modulation.pitch_interval |= 0x80;
                    modulation.pitch_phase ^= 0x80;
                } else {
                    modulation.pitch_interval &= 0x7f;
                }
            }
        }
        let output = period.wrapping_add(modulation.pitch_offset);
        let [mut low, mut high] = output.to_le_bytes();
        if voice == Voice::Noise {
            let control = if secondary {
                channel.secondary.control
            } else {
                channel.primary.control
            };
            low = (low & 15) | (control & 0x80);
            high = 0;
            next_selector = Some(SeedSelector {
                selector: control & 0x80,
                producer: SeedProducer::NoiseMode,
            });
        }
        let base = 0x4000 + voice.index() as u16 * 4;
        let mut writes = vec![RegisterWrite {
            address: base + 2,
            value: low,
        }];
        if high != modulation.cached_high {
            modulation.cached_high = high;
            writes.push(RegisterWrite {
                address: base + 3,
                value: high | 8,
            });
        }
        if let Some(selector) = next_selector {
            self.reset_selector = selector;
        }
        writes
    }
    pub fn reset_modulation(&mut self, voice: Voice, seed: ResetLevel, reset: Reset) {
        let modulation = &mut self.channels[voice.index()].modulation;
        let cached = modulation.cached_high;
        modulation.pitch_interval &= 0x7f;
        if modulation.level_interval & 0x80 != 0 {
            modulation.level = if voice == Voice::Triangle {
                seed.value
            } else {
                seed.value & 15
            };
        }
        modulation.level_interval &= 0x7f;
        modulation.pitch_counter = 0;
        modulation.pitch_phase = 0;
        modulation.pitch_offset = 0;
        modulation.level_counter = 0;
        modulation.cached_high = if reset == Reset::KeepPeriod {
            cached
        } else {
            0xff
        };
    }
    pub fn load_preset(&mut self, catalog: &Catalog, voice: Voice, index: u8) {
        let address = self.install_preset_fields(catalog, voice, index);
        self.select_seed(address.wrapping_add(3) as u8, SeedProducer::PresetPointer);
    }
    fn install_preset_fields(&mut self, catalog: &Catalog, voice: Voice, index: u8) -> u16 {
        let address = self
            .preset_base
            .wrapping_add(u16::from(index.wrapping_mul(4)));
        let read = |offset| {
            catalog
                .read(address.wrapping_add(offset))
                .expect("validated audio preset")
        };
        let modulation = &mut self.channels[voice.index()].modulation;
        modulation.pitch_interval = read(0);
        modulation.pitch_shape = read(1);
        modulation.level_interval = read(2);
        modulation.level_step = read(3) as i8;
        address
    }
}
/// The APU is the toolkit's independent MIT mechanism. Driver calls schedule
/// register changes; sample requests only advance sound clocks, never the driver.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Synthesizer {
    pub apu: Apu,
    pub elapsed_clocks: u64,
    /// Fractional NTSC CPU clocks retained when using Apu::clock/output.
    /// Apu::render also owns this phase internally; implementation must choose
    /// one sample-clock owner, never reset it at host callback boundaries.
    pub sample_clock_fraction: f64,
}
impl Default for Synthesizer {
    fn default() -> Self {
        Self {
            apu: Apu::default(),
            elapsed_clocks: 0,
            sample_clock_fraction: 0.0,
        }
    }
}
impl Synthesizer {
    pub fn apply(&mut self, writes: &[RegisterWrite]) {
        for write in writes {
            self.apu
                .write(write.address, write.value)
                .expect("driver APU register");
        }
    }
    pub fn render(&mut self, output: &mut [f32], sample_rate: u32) {
        assert!((8000..=192000).contains(&sample_rate));
        for sample in output {
            self.sample_clock_fraction += 1_789_773.0 / f64::from(sample_rate);
            let clocks = self.sample_clock_fraction.floor() as u64;
            self.sample_clock_fraction -= clocks as f64;
            let mut sum = 0.0;
            for _ in 0..clocks {
                self.apu.clock(&mut |_| 0);
                sum += self.apu.output();
            }
            self.elapsed_clocks += clocks;
            *sample = sum / clocks as f32;
        }
    }
}
#[cfg(test)]
mod contracts;
