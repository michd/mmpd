pub mod midi;

use midi::MidiPrecondition;

#[derive(PartialEq, Debug)]
pub enum Precondition {
    Midi(MidiPrecondition),
    Other // Placeholder
}