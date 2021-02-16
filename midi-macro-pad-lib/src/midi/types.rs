use std::result;
use num_derive::FromPrimitive;

#[derive(FromPrimitive)]
enum ChannelMessageType {
    NoteOff = 0b1000isize,
    NoteOn = 0b1001isize,
    PolyAftertouch = 0b1010isize,
    ControlChange = 0b1011isize,
    ProgramChange = 0b1100isize,
    ChannelAfterTouch = 0b1101isize,
    PitchBendChange = 0b1110isize,
    System = 0b1111isize,
}

pub enum Error {
    FailedToRead
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum MidiMessage {
    NoteOff { channel: u8, key: u8, velocity: u8 },
    NoteOn { channel: u8, key: u8, velocity: u8 },
    PolyAftertouch { channel: u8, key: u8, value: u8 },
    ControlChange { channel: u8, control: u8, value: u8 },
    ProgramChange { channel: u8, program: u8 },
    ChannelAftertouch { channel: u8, value: u8 },
    PitchBendChange { channel: u8, value: u16 },
    //TimingClock,
    //Start,
    //Continue,
    //Stop,
    Other
}
