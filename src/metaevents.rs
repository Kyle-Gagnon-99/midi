use std::time::Duration;

use crate::{midi_error::MidiError, metadata::TimeDivision};

// Use re-exports so that it is easier to grab the events
mod sequence_number;
pub use sequence_number::SequenceNumber;

mod text;
pub use text::TextEvent;

mod copyright_notice;
pub use copyright_notice::CopyRightNoticeEvent;

mod track_name;
pub use track_name::TrackNameEvent;

mod instrument_name;
pub use instrument_name::InstrumentNameEvent;

mod lyric;
pub use lyric::LyricEvent;

mod marker;
pub use marker::MarkerEvent;

mod cue_point;
pub use cue_point::CuePointEvent;

mod midi_channel_prefix;
pub use midi_channel_prefix::MidiChannelPrefixEvent;

mod set_tempo;
pub use set_tempo::SetTempoEvent;

mod time_signature;
pub use time_signature::{TimeSignatureEvent, TimeSignature};

mod end_of_track;
pub use end_of_track::EndOfTrack;

mod key_signature;
pub use key_signature::KeySignatureEvent;
pub use key_signature::KeySignature;

mod smpte_offset;
pub use smpte_offset::SMPTEOffsetEvent;

mod sequencer_specific;
pub use sequencer_specific::SequencerSpecificEvent;

mod midi_port;
pub use midi_port::MidiPortEvent;

const METAEVENT_BYTE: u8 = 0xFF;

/// Given a vector of bytes convert the data from bytes to UTF-8
/// The first byte is the length of the data and then the bytes
/// following should be the data
/// 
/// Returns the length of the data and a string of the data
/// 
/// # Arguments
/// * `data: &[u8]` - The set of bytes to extract
pub(crate) fn get_utf8_from_bytes(data: &[u8]) -> Result<String, MidiError> {
    // Convert from the byte after the length byte to the length to UTF8
    let utf8_data = String::from_utf8((data).to_vec())?;

    Ok(utf8_data)
}

/// Converts a delta time in ticks to the variable-length quantity (VLQ) format used in MIDI.
/// 
/// # Arguments
/// 
/// * `delta_time` - A `u32` value representing the delta time in ticks.
/// 
/// # Returns
/// 
/// A `Vec` of `u8` bytes representing the delta time in the variable-length quantity (VLQ) format.
pub(crate) fn from_vlq_to_bytes(delta_time: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut value = delta_time;

    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value > 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }

    bytes
}

/// Given an array of u8 bytes, find out the delta time of variable-length format
/// Variable-length format in essence says that if the first bit of the byte (MSB)
/// is a 1 then the next byte is a part of the delta time. Otherwise that byte is a
/// part of delta time. Then the rest of the 7 bits that are apart of the byte are used
/// to calculate the delta time.
/// 
/// # Arguments
/// * `track_data` &[[u8]] - The data of the track. Should be at max 5 bytes
pub(crate) fn from_bytes_to_vlq(track_data: &[u8]) -> (u32, u8) {
    let mut num_of_bytes: u8 = 0;
    let mut delta_time: u32 = 0;
    let mut shift: u32 = 0;

    for byte in track_data {
        delta_time |= ((byte & 0b0111_1111) as u32) << shift;
        num_of_bytes += 1;

        if byte & 0b1000_0000 == 0 {
            break;
        }

        shift += 7;
    }

    (delta_time, num_of_bytes)
}

/// Calculates the time duration based on the provided delta time, time division, and tempo.
///
/// # Arguments
///
/// * `delta_time` - a 32-bit unsigned integer representing the time duration in ticks
/// * `time_division` - an enum representing either pulses per quarter note or SMPTE (Society of Motion Picture and Television Engineers) timecode
/// * `tempo` - a 32-bit unsigned integer representing the tempo in beats per minute
///
/// # Returns
///
/// A `Duration` object representing the calculated time duration.
pub(crate) fn calculate_time_duration(delta_time: u32, time_division: TimeDivision, tempo: u32) -> Duration {
    let ticks_per_quarter_note: u16 = match time_division {
        TimeDivision::PulsesPerQuarterNote(pulses) => pulses,
        TimeDivision::SMPTE(fps, ticks_per_frame) => (fps * ticks_per_frame / 4) as u16,
    };

    let microseconds_per_quarter_note = 60_000_000 / tempo;
    let seconds_per_quarter_note = microseconds_per_quarter_note as f32 / 1_000_000.0;
    let time_duration_ticks = delta_time as f32 / ticks_per_quarter_note as f32;
    let time_duration_seconds = time_duration_ticks * seconds_per_quarter_note;

    Duration::from_secs_f32(time_duration_seconds)
}

/// Converts a duration in microseconds to beats per minute (BPM).
///
/// # Arguments
///
/// * `microseconds` - a 32-bit unsigned integer representing the duration in microseconds
///
/// # Returns
///
/// A `f64` value representing the calculated BPM.
pub(crate) fn microseconds_to_bpm(microseconds: u32) -> f64 {
    60_000_000.0 / (microseconds as f64)
}

/// Converts a BPM value to a duration in microseconds.
///
/// # Arguments
///
/// * `bpm` - a `f64` value representing the tempo in beats per minute
///
/// # Returns
///
/// A 32-bit unsigned integer representing the calculated duration in microseconds.
pub(crate) fn bpm_to_microseconds(bpm: f64) -> u32 {
    (60_000_000.0 / bpm).round() as u32
}

#[cfg(test)]
mod metaevent_tests {
    use super::*;

    #[test]
    fn get_utf8_bytes_success() {
        let test_text = String::from("This is some testing text!! Yay! 1234567890,/\\!@#$%^&*()';\"[]");

        let mut data: Vec<u8> = vec![];
        data.extend_from_slice(test_text.as_bytes());

        let result = get_utf8_from_bytes(&data);
        assert!(result.is_ok());

        let text_data = result.unwrap();
        assert_eq!(text_data, test_text);
    }
}