use log::debug;

#[cfg(feature = "json")]
use serde::Serialize;

use crate::{
    events::{dispatch_from_bytes, Event, SerializableEvent},
    metadata::TimeDivision,
    metaevents::{
        from_bytes_to_vlq, CopyRightNoticeEvent, CuePointEvent, EndOfTrack, InstrumentNameEvent,
        KeySignatureEvent, LyricEvent, MarkerEvent, MidiChannelPrefixEvent, SMPTEOffsetEvent,
        SequenceNumber, SequencerSpecificEvent, SetTempoEvent, TextEvent, TimeSignatureEvent,
        TrackNameEvent, MidiPortEvent,
    },
    midi_error::{MidiError, ParseError},
    TimeSignature, messages::{NoteOnEvent, EVENT_MASK, NoteOffEvent, ControlChangeEvent, PolyphonicKeyPressureEvent, ProgramChangeEvent, ChannelPressureEvent, PitchBendChangeEvent}, print_file_contents, is_msb_zero,
};

const TRACK_HEADER_BYTES: u32 = u32::from_be_bytes([0x4D, 0x54, 0x72, 0x6B]);

#[derive(Debug)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct Track {
    pub events: Vec<SerializableEvent>,
    pub track_size: u32,
}

impl Track {
    fn new(
        data: &[u8],
        time_division: TimeDivision,
        tempo: u32,
        time_signature: TimeSignature,
    ) -> Result<(Self, u32, TimeSignature), MidiError> {
        // Validate that this is the start of a track
        debug!("The track data is as follows");
        print_file_contents(data);
        validate_track_header_chunk_bytes(u32::from_be_bytes([
            data[0], data[1], data[2], data[3],
        ]))?;

        // Now get the size of the track
        let track_size: u32 = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        debug!("The track size is {}", track_size);

        // The new data is from byte 8 to the size of the track
        let data = &data[8..(track_size + 8) as usize];

        // Set up an empty event list
        let mut events: Vec<SerializableEvent> = Vec::new();

        let mut position = 0;

        let mut running_status = 0;
        let mut channel = 0;

        while position < track_size {

            let (delta_time, delta_time_bytes) = from_bytes_to_vlq(&data[position as usize..]);
            debug!("The delta time took {} byte(s)", delta_time_bytes);
            debug!("The delta time was {}", delta_time);
            position += delta_time_bytes as u32;

            print_file_contents(&data[position as usize..]);

            // If the two bytes next to each other both have their MSB as 0 that means they are data bytes a part of the current running status
            if is_msb_zero(data[position as usize]) && is_msb_zero(data[(position + 1) as usize]) && running_status != 0 {
                debug!("Byte {:02X} and byte {:02X} are probably data bytes to the current running status {:2X}", data[position as usize], data[(position + 1) as usize], running_status);

                // We should instead get the event with the running status with the correct data and then increment the position by two and then continue the loop
                let event = parse_running_status_data(running_status, channel, data, delta_time, time_division, tempo)?;
                debug!("Added {} to the event list with the data bytes!", event.get_event_name());

                events.push(SerializableEvent(event));

                position += 2;
                continue;
            }

            let event = parse_event(data[position as usize], &data[position as usize..], delta_time, time_division, tempo)?;
            position += event.get_event_size() as u32;

            debug!("Got {} event!", event.get_event_name());
            debug!("The event is {} bytes long", event.get_event_size());

            if event.is_running_status_allowed() {
                debug!("The event supports the running status");
                running_status = event.event_type();
                channel = event.get_channel();
            }

            events.push(SerializableEvent(event));
        }

        Ok((
            Self {
                events,
                track_size,
            },
            tempo,
            time_signature,
        ))
    }

    pub fn get_track_list(data: &[u8]) -> Result<Vec<Self>, MidiError> {
        // Create an empty list of tracks
        let mut track_list: Vec<Self> = Vec::new();

        // Create the default time signature (4/4)
        let mut time_signature = TimeSignature::new(4, 4)?;

        // Set the default tempo (120 BPM)
        let mut tempo: u32 = 120;

        let mut position = 0;

        while position < data.len() {
            // Now create each track; We give each track the tempo and time signature in case it overrides the overall time signature and/or tempo
            let (track, tempo_override, time_signature_override) = Self::new(&data[position..], TimeDivision::PulsesPerQuarterNote(96), tempo, time_signature)?;
            tempo = tempo_override;
            time_signature = time_signature_override;
            position += (track.track_size + 8) as usize;

            // Push the track to the list
            track_list.push(track);
        }

        // Return the list of tracks and any tempo / time signature changes
        Ok(track_list)
    }
}

fn parse_event(status_byte: u8, data: &[u8], delta_time: u32, time_division: TimeDivision, tempo: u32) -> Result<Box<dyn Event>, MidiError> {
    match status_byte {
        0xFF => parse_meta_event(data, delta_time, time_division, tempo),
        _ => {
                let event_type = (status_byte & EVENT_MASK) >> 4;
                match event_type {
                    0x9 => dispatch_from_bytes::<NoteOnEvent>(data, delta_time, time_division, tempo),
                    0x8 => dispatch_from_bytes::<NoteOffEvent>(data, delta_time, time_division, tempo),
                    0xA => dispatch_from_bytes::<PolyphonicKeyPressureEvent>(data, delta_time, time_division, tempo),
                    0xB => dispatch_from_bytes::<ControlChangeEvent>(data, delta_time, time_division, tempo),
                    0xC => dispatch_from_bytes::<ProgramChangeEvent>(data, delta_time, time_division, tempo),
                    0xD => dispatch_from_bytes::<ChannelPressureEvent>(data, delta_time, time_division, tempo),
                    0xE => dispatch_from_bytes::<PitchBendChangeEvent>(data, delta_time, time_division, tempo),
                    _ => Err(MidiError::ParseError(ParseError::NotImplemented(format!("Event {:02X} is not implemented!", event_type)))),
                }
            }
    }
}

fn parse_running_status_data(running_status: u8, channel: u8, data: &[u8], delta_time: u32, time_division: TimeDivision, tempo: u32) -> Result<Box<dyn Event>, MidiError> {
    match running_status >> 4 {
        0x9 => Ok(Box::new(NoteOnEvent::new_from_status(data, delta_time, channel, time_division, tempo)?)),
        0xB => Ok(Box::new(ControlChangeEvent::new_from_status(data, channel, time_division, tempo)?)),
        _ => Err(MidiError::ParseError(ParseError::InvalidEventBytes(format!("Invalid running status!")))),
    }
}

fn parse_meta_event(
    data: &[u8],
    delta_time: u32,
    ticks_per_quarter_note: TimeDivision,
    tempo: u32,
) -> Result<Box<dyn Event>, MidiError> {
    let returned_event: Box<dyn Event> = match data[1] {
        0x00 => {
            dispatch_from_bytes::<SequenceNumber>(data, delta_time, ticks_per_quarter_note, tempo)
        }
        0x01 => dispatch_from_bytes::<TextEvent>(data, delta_time, ticks_per_quarter_note, tempo),
        0x02 => dispatch_from_bytes::<CopyRightNoticeEvent>(
            data,
            delta_time,
            ticks_per_quarter_note,
            tempo,
        ),
        0x03 => {
            dispatch_from_bytes::<TrackNameEvent>(data, delta_time, ticks_per_quarter_note, tempo)
        }
        0x04 => dispatch_from_bytes::<InstrumentNameEvent>(
            data,
            delta_time,
            ticks_per_quarter_note,
            tempo,
        ),
        0x05 => dispatch_from_bytes::<LyricEvent>(data, delta_time, ticks_per_quarter_note, tempo),
        0x06 => dispatch_from_bytes::<MarkerEvent>(data, delta_time, ticks_per_quarter_note, tempo),
        0x07 => {
            dispatch_from_bytes::<CuePointEvent>(data, delta_time, ticks_per_quarter_note, tempo)
        }
        0x20 => dispatch_from_bytes::<MidiChannelPrefixEvent>(
            data,
            delta_time,
            ticks_per_quarter_note,
            tempo,
        ),
        0x21 => dispatch_from_bytes::<MidiPortEvent>(data, delta_time, ticks_per_quarter_note, tempo),
        0x2F => dispatch_from_bytes::<EndOfTrack>(data, delta_time, ticks_per_quarter_note, tempo),
        0x51 => {
            dispatch_from_bytes::<SetTempoEvent>(data, delta_time, ticks_per_quarter_note, tempo)
        }
        0x54 => {
            dispatch_from_bytes::<SMPTEOffsetEvent>(data, delta_time, ticks_per_quarter_note, tempo)
        }
        0x58 => dispatch_from_bytes::<TimeSignatureEvent>(
            data,
            delta_time,
            ticks_per_quarter_note,
            tempo,
        ),
        0x59 => dispatch_from_bytes::<KeySignatureEvent>(
            data,
            delta_time,
            ticks_per_quarter_note,
            tempo,
        ),
        0x7F => dispatch_from_bytes::<SequencerSpecificEvent>(
            data,
            delta_time,
            ticks_per_quarter_note,
            tempo,
        ),
        _ => {
            return Err(MidiError::ParseError(ParseError::NotImplemented(
                String::from(format!("{:02X} is not an implemented Meta Event", data[1])),
            )))
        }
    }?;
    Ok(returned_event)
}

fn validate_track_header_chunk_bytes(track_header_chunk_bytes: u32) -> Result<(), MidiError> {
    match track_header_chunk_bytes {
        TRACK_HEADER_BYTES => Ok(()),
        _ => Err(MidiError::ParseError(ParseError::InvalidHeader)),
    }
}
