pub mod midi;

use midi::MidiPrecondition;

#[derive(PartialEq, Debug)]
pub struct Precondition {
    pub invert: bool,
    pub condition: PreconditionType
}

#[derive(PartialEq, Debug)]
pub enum PreconditionType {
    Midi(MidiPrecondition),
    Other // Placeholder
}