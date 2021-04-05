pub mod midi;

use midi::MidiPrecondition;

#[derive(PartialEq, Debug)]
pub struct Precondition {
    pub invert: bool,
    pub condition: PreconditionType
}

// TODO: turn back into a struct so it can have an `invert` field
#[derive(PartialEq, Debug)]
pub enum PreconditionType {
    Midi(MidiPrecondition),
    Other // Placeholder
}