use std::{
    time::{Duration, Instant},
};

#[cfg(feature = "json")]
use serde::Serialize;

use crate::{
    events::{Event, FromBytes},
    metadata::TimeDivision,
    metaevents::{calculate_time_duration, from_vlq_to_bytes},
    midi_error::MidiError,
    note::Note,
};

use super::CHANNEL_MASK;

// This an event type which means its the first 4 bits in the first byte
// After that the channel bits are followed to create the fully status byte
const MIDI_EVENT_TYPE: u8 = 0x90;

const NOTE_ON_SIZE: u8 = 0x03;

const EVENT_NAME: &str = "Note On";

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct NoteOnEvent {
    pub midi_note: u8,
    pub velocity: u8,
    pub channel: u8,
    event_name: String,
    event_size: u8,
    delta_time: u32,
    
    #[cfg_attr(feature = "json", serde(skip))]
    current_time: Instant,
    time_duration: Duration,
}

impl Event for NoteOnEvent {
    fn get_event_name(&self) -> String {
        self.event_name.to_owned()
    }

    fn is_running_status_allowed(&self) -> bool {
        true
    }

    fn event_type(&self) -> u8 {
        MIDI_EVENT_TYPE
    }

    fn get_channel(&self) -> u8 {
        self.channel
    }

    fn get_event_size(&self) -> u8 {
        self.event_size
    }

    fn get_delta_time(&self) -> u32 {
        self.delta_time
    }

    fn get_current_time(&self) -> Instant {
        self.current_time
    }

    fn get_time_duration(&self) -> Duration {
        self.time_duration
    }

    fn to_bytes(&self) -> Result<Vec<u8>, MidiError> {
        let bytes: Vec<u8> = vec![
            (MIDI_EVENT_TYPE | self.channel),
            self.midi_note,
            self.velocity,
        ];

        Ok(bytes)
    }

    fn to_bytes_delta_time(&self) -> Result<Vec<u8>, MidiError> {
        let mut bytes: Vec<u8> = from_vlq_to_bytes(self.delta_time);
        bytes.extend_from_slice(&[
            (MIDI_EVENT_TYPE | self.channel),
            self.midi_note,
            self.velocity,
        ]);

        Ok(bytes)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl FromBytes for NoteOnEvent {
    type Output = Self;

    fn from_bytes(
        data: &[u8],
        delta_time: u32,
        time_division: TimeDivision,
        tempo: u32,
    ) -> Result<Self::Output, MidiError> {
        // Calculate the time duration
        let time_duration = calculate_time_duration(delta_time, time_division, tempo);

        // The data starts at the beginning
        let data = &data[0..];

        // The first byte is the status byte
        // The first four bits say what event this is (which we don't care about now)
        // then the last four bits give us the channel number
        let channel = data[0] & CHANNEL_MASK;

        // The next byte gets the note number (between 0 - 127)
        let midi_note = data[1];

        // The final byte gets the velocity (between 0 - 127)
        let velocity = data[2];

        Ok(Self {
            midi_note,
            velocity,
            channel,
            event_name: EVENT_NAME.to_string(),
            event_size: NOTE_ON_SIZE,
            delta_time,
            current_time: Instant::now(),
            time_duration,
        })
    }
}

impl<'a> NoteOnEvent {
    pub fn new(midi_note: u8, velocity: u8, channel: u8) -> Result<Self, MidiError> {
        Ok(Self {
            midi_note,
            velocity,
            channel,
            event_name: EVENT_NAME.to_string(),
            event_size: NOTE_ON_SIZE,
            delta_time: 0,
            current_time: Instant::now(),
            time_duration: Duration::from_secs(0),
        })
    }

    pub fn new_from_note(note: Note<'a>, velocity: u8, channel: u8) -> Result<Self, MidiError> {
        Ok(Self {
            midi_note: note.to_midi_note(),
            velocity,
            channel,
            event_name: EVENT_NAME.to_string(),
            event_size: NOTE_ON_SIZE,
            delta_time: 0,
            current_time: Instant::now(),
            time_duration: Duration::from_secs(0),
        })
    }

    pub fn get_note(&self) -> Note {
        Note::new_from_midi_note(self.midi_note)
    }

    pub fn new_from_status(data: &[u8], delta_time: u32, channel: u8, time_division: TimeDivision, tempo: u32) -> Result<Self, MidiError> {
        // Calculate the time duration
        let time_duration = calculate_time_duration(delta_time, time_division, tempo);

        Ok(
            Self {
                midi_note: data[0],
                velocity: data[1],
                channel,
                event_name: EVENT_NAME.to_string(),
                event_size: NOTE_ON_SIZE,
                delta_time,
                current_time: Instant::now(),
                time_duration,
            }
        )
    }
}
