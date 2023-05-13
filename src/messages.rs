pub const CHANNEL_MASK: u8 = 0b0000_1111;
pub(crate) const EVENT_MASK: u8 = 0b1111_0000;

// Use re-exports so that it is easier to grab the events
mod note_on;
pub use note_on::NoteOnEvent;

mod note_off;
pub use note_off::NoteOffEvent;

mod control_change;
pub use control_change::ControlChangeEvent;

mod polyphonic_key_pressure;
pub use polyphonic_key_pressure::PolyphonicKeyPressureEvent;

mod program_change;
pub use program_change::ProgramChangeEvent;

mod channel_pressure;
pub use channel_pressure::ChannelPressureEvent;

mod pitch_bend_change;
pub use pitch_bend_change::PitchBendChangeEvent;