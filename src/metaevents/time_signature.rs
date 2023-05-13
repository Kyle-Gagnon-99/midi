use std::{
    time::{Duration, Instant},
};

#[cfg(feature = "json")]
use serde::Serialize;

use crate::{
    events::{Event, FromBytes},
    metadata::TimeDivision,
    midi_error::MidiError,
};

use super::{calculate_time_duration, from_vlq_to_bytes, METAEVENT_BYTE};

const METAEVENT_BYTE_TYPE: u8 = 0x58;

const TIME_SIGNATURE_SIZE: u8 = 7;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct TimeSignature {
    pub numerator: u8,
    pub denominator: u8,
}

impl TimeSignature {
    pub fn new(numerator: u8, denominator: u8) -> Result<Self, MidiError> {
        Ok(Self {
            numerator,
            denominator,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct TimeSignatureEvent {
    pub numerator: u8,
    pub denominator: u8,
    pub metronome_clicks: u8,
    pub num_of_32nd_notes_per_quarter: u8,
    event_size: u8,
    delta_time: u32,

    #[cfg_attr(feature = "json", serde(skip))]
    current_time: Instant,
    time_duration: Duration,
}

impl Event for TimeSignatureEvent {
    fn get_event_name(&self) -> String {
        String::from("Time Signature")
    }

    fn is_running_status_allowed(&self) -> bool {
        false
    }

    fn event_type(&self) -> u8 {
        METAEVENT_BYTE_TYPE
    }

    fn get_channel(&self) -> u8 {
        0
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
        let mut bytes: Vec<u8> = vec![
            METAEVENT_BYTE,
            METAEVENT_BYTE_TYPE,
            (TIME_SIGNATURE_SIZE - 3),
        ];
        bytes.extend_from_slice(&[
            self.numerator,
            (self.denominator as f64).log2() as u8,
            self.metronome_clicks,
            self.num_of_32nd_notes_per_quarter,
        ]);

        Ok(bytes)
    }

    fn to_bytes_delta_time(&self) -> Result<Vec<u8>, MidiError> {
        let mut bytes: Vec<u8> = from_vlq_to_bytes(self.delta_time);
        bytes.extend_from_slice(&[
            METAEVENT_BYTE,
            METAEVENT_BYTE_TYPE,
            (TIME_SIGNATURE_SIZE - 3),
        ]);
        bytes.extend_from_slice(&[
            self.numerator,
            (self.denominator as f64).log2() as u8,
            self.metronome_clicks,
            self.num_of_32nd_notes_per_quarter,
        ]);

        Ok(bytes)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl FromBytes for TimeSignatureEvent {
    type Output = Self;

    fn from_bytes(
        data: &[u8],
        delta_time: u32,
        time_division: TimeDivision,
        tempo: u32,
    ) -> Result<Self::Output, MidiError> {
        // Calculate the time duration
        let time_duration = calculate_time_duration(delta_time, time_division, tempo);

        // The data starts after 0xFF 0x58 0x04
        let data = &data[3..];

        // The next byte is the numerator
        let numerator = data[0];

        // The next byte is the denominator (which is what we raise the power of two to)
        // 2 represents a quarter note, 3 represents an eight note, and son on
        let denominator: u8 = 2_u8.pow(data[1] as u32);

        // The next byte is the number of MIDI clocks per metronome click
        let metronome_clicks = data[2];

        // The next byte tells MIDI how many 32nd notes are in a quarter note (typically should be 8)
        let num_of_32nd_notes_per_quarter = data[3];

        Ok(Self {
            numerator,
            denominator,
            metronome_clicks,
            num_of_32nd_notes_per_quarter,
            event_size: TIME_SIGNATURE_SIZE,
            delta_time,
            current_time: Instant::now(),
            time_duration,
        })
    }
}

impl TimeSignatureEvent {
    pub fn new(numerator: u8, denominator: u8) -> Result<Self, MidiError> {
        Ok(Self {
            numerator,
            denominator,
            metronome_clicks: 96,
            num_of_32nd_notes_per_quarter: 8,
            event_size: TIME_SIGNATURE_SIZE,
            delta_time: 0,
            current_time: Instant::now(),
            time_duration: Duration::from_secs(0),
        })
    }
}
