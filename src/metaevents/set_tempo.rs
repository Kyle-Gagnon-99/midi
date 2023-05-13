use std::time::{Duration, Instant};

#[cfg(feature = "json")]
use serde::Serialize;

use crate::{
    events::{Event, FromBytes},
    metadata::TimeDivision,
    midi_error::MidiError,
};

use super::{
    bpm_to_microseconds, calculate_time_duration, from_vlq_to_bytes, microseconds_to_bpm,
    METAEVENT_BYTE,
};

const METAEVENT_BYTE_TYPE: u8 = 0x51;

const SET_TEMPO_SIZE: u8 = 6;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct SetTempoEvent {
    pub tempo: f64,
    event_size: u8,
    delta_time: u32,

    #[cfg_attr(feature = "json", serde(skip))]
    current_time: Instant,
    time_duration: Duration,
}

impl Event for SetTempoEvent {
    fn get_event_name(&self) -> String {
        String::from("Set Tempo")
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
        let mut bytes: Vec<u8> = vec![METAEVENT_BYTE, METAEVENT_BYTE_TYPE, (SET_TEMPO_SIZE - 3)];
        let bpm_to_microseconds = bpm_to_microseconds(self.tempo);
        let bpm_to_microseconds = bpm_to_microseconds.to_be_bytes();
        bytes.extend_from_slice(&[
            bpm_to_microseconds[1],
            bpm_to_microseconds[2],
            bpm_to_microseconds[3],
        ]);

        Ok(bytes)
    }

    fn to_bytes_delta_time(&self) -> Result<Vec<u8>, MidiError> {
        let mut bytes: Vec<u8> = from_vlq_to_bytes(self.delta_time);
        bytes.extend_from_slice(&[METAEVENT_BYTE, METAEVENT_BYTE_TYPE, (SET_TEMPO_SIZE - 3)]);
        let bpm_to_microseconds = bpm_to_microseconds(self.tempo);
        let bpm_to_microseconds = bpm_to_microseconds.to_be_bytes();
        bytes.extend_from_slice(&[
            bpm_to_microseconds[1],
            bpm_to_microseconds[2],
            bpm_to_microseconds[3],
        ]);

        Ok(bytes)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl FromBytes for SetTempoEvent {
    type Output = Self;

    fn from_bytes(
        data: &[u8],
        delta_time: u32,
        time_division: TimeDivision,
        tempo: u32,
    ) -> Result<Self::Output, MidiError> {
        // Calculate time duration
        let time_duration = calculate_time_duration(delta_time, time_division, tempo);

        // The data starts after 0xFF 0x51 0x03
        let data = &data[3..];

        // The tempo is in microseconds for the next three bytes
        let tempo_in_micro = u32::from_be_bytes([0, data[0], data[1], data[2]]);

        // Convert the tempo in microseconds to actual BPM
        let tempo = microseconds_to_bpm(tempo_in_micro);

        // Construct the struct
        Ok(Self {
            tempo,
            event_size: SET_TEMPO_SIZE,
            delta_time,
            time_duration,
            current_time: Instant::now(),
        })
    }
}

impl SetTempoEvent {
    pub fn new(tempo: f64) -> Result<Self, MidiError> {
        Ok(Self {
            tempo,
            event_size: SET_TEMPO_SIZE,
            delta_time: 0,
            current_time: Instant::now(),
            time_duration: Duration::from_secs(0),
        })
    }
}

#[cfg(test)]
mod set_tempo_tests {
    use crate::{
        events::{Event, FromBytes},
        metadata::TimeDivision,
        metaevents::bpm_to_microseconds,
    };

    use super::SetTempoEvent;

    #[test]
    fn create_tempo_event_success() {
        // Set the tempo to 96 BPM
        let tempo = 96.0;

        let tempo_event = SetTempoEvent::new(tempo);
        assert!(tempo_event.is_ok());

        let tempo_event = tempo_event.unwrap();
        assert_eq!(tempo_event.tempo, tempo);
    }

    #[test]
    fn create_tempo_event_success_to_bytes() {
        // Set the tempo to 96 BPM
        let tempo = 96.0;

        // Assert there is no errors creating it
        let tempo_event = SetTempoEvent::new(tempo);
        assert!(tempo_event.is_ok());

        // Assert that the tempo is what we gave it
        let tempo_event = tempo_event.unwrap();
        assert_eq!(tempo_event.tempo, tempo);

        // Convert the event to bytes
        let tempo_event_bytes = tempo_event.to_bytes();
        assert!(tempo_event_bytes.is_ok());

        let tempo_event_bytes = tempo_event_bytes.unwrap();

        // Construct what we expect for the bytes
        let tempo_bpm_to_microseconds = bpm_to_microseconds(tempo);
        let tempo_bpm_to_microseconds = tempo_bpm_to_microseconds.to_be_bytes();
        let tempo_bpm_to_microseconds = [
            tempo_bpm_to_microseconds[1],
            tempo_bpm_to_microseconds[2],
            tempo_bpm_to_microseconds[3],
        ];

        let mut tempo_event_bytes_expected = vec![0xFF, 0x51, 0x03];
        tempo_event_bytes_expected.extend_from_slice(&tempo_bpm_to_microseconds);

        assert_eq!(tempo_event_bytes, tempo_event_bytes_expected);
    }

    #[test]
    fn create_tempo_event_success_from_bytes() {
        // Set the tempo
        let tempo = 96.0;

        // Get the bytes to represent the tempo
        let tempo_bpm_to_microseconds = bpm_to_microseconds(tempo);
        let tempo_bpm_to_microseconds = tempo_bpm_to_microseconds.to_be_bytes();
        let tempo_bpm_to_microseconds = [
            tempo_bpm_to_microseconds[1],
            tempo_bpm_to_microseconds[2],
            tempo_bpm_to_microseconds[3],
        ];

        let mut tempo_event_bytes = vec![0xFF, 0x51, 0x03];
        tempo_event_bytes.extend_from_slice(&tempo_bpm_to_microseconds);

        let tempo_event = SetTempoEvent::from_bytes(
            &tempo_event_bytes,
            0,
            TimeDivision::PulsesPerQuarterNote(96),
            tempo as u32,
        );
        assert!(tempo_event.is_ok());

        let tempo_event = tempo_event.unwrap();
        assert_eq!(tempo_event.tempo, tempo);
    }
}
