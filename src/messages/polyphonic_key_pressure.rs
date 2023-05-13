use std::time::{Duration, Instant};

#[cfg(feature = "json")]
use serde::Serialize;

use crate::{
    events::{Event, FromBytes},
    metadata::TimeDivision,
    metaevents::{calculate_time_duration, from_vlq_to_bytes},
    midi_error::MidiError,
};

use super::CHANNEL_MASK;

const MIDI_EVENT_TYPE: u8 = 0xA0;

const POLYPHONIC_KEY_PRESSURE_SIZE: u8 = 0x03;

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct PolyphonicKeyPressureEvent {
    pub midi_note: u8,
    pub control_pressure: u8,
    pub channel: u8,
    event_size: u8,
    delta_time: u32,

    #[cfg_attr(feature = "json", serde(skip))]
    current_time: Instant,
    time_duration: Duration,
}

impl Event for PolyphonicKeyPressureEvent {
    fn get_event_name(&self) -> String {
        String::from("Polyphonic Key Pressure")
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
            self.control_pressure,
        ];

        Ok(bytes)
    }

    fn to_bytes_delta_time(&self) -> Result<Vec<u8>, MidiError> {
        let mut bytes: Vec<u8> = from_vlq_to_bytes(self.delta_time);
        bytes.extend_from_slice(&[
            (MIDI_EVENT_TYPE | self.channel),
            self.midi_note,
            self.control_pressure,
        ]);

        Ok(bytes)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl FromBytes for PolyphonicKeyPressureEvent {
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
        // The first four bits (nibble) say what the event is
        // THe last four bits say what channel
        let channel = data[0] & CHANNEL_MASK;

        // The next byte is what midi note this is applied to
        let midi_note = data[1];

        // The final byte is the value of the pressure
        let control_pressure = data[2];

        Ok(Self {
            midi_note,
            control_pressure,
            channel,
            event_size: POLYPHONIC_KEY_PRESSURE_SIZE,
            delta_time,
            current_time: Instant::now(),
            time_duration,
        })
    }
}
