#[derive(Debug, Eq, PartialEq)]
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
