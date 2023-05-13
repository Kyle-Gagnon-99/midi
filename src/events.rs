use std::{
    any::Any,
    fmt::Debug,
    time::{Duration, Instant},
};

#[cfg(feature = "json")]
use serde::Serialize;

use crate::{metadata::TimeDivision, midi_error::MidiError};

#[cfg(feature = "json")]
use crate::{
    messages::{
        ChannelPressureEvent, ControlChangeEvent, PitchBendChangeEvent, PolyphonicKeyPressureEvent,
        ProgramChangeEvent, NoteOffEvent, NoteOnEvent
    },
    metaevents::{
        CopyRightNoticeEvent, CuePointEvent, EndOfTrack, InstrumentNameEvent, KeySignatureEvent,
        LyricEvent, MarkerEvent, MidiChannelPrefixEvent, MidiPortEvent, SMPTEOffsetEvent,
        SequenceNumber, SequencerSpecificEvent, SetTempoEvent, TextEvent, TimeSignatureEvent,
        TrackNameEvent,
    },
};

pub trait Event: Debug + Any {
    /// Gets the name of the event. Mainly used for debugging.
    ///
    /// # Return
    /// Returns the name of the event
    fn get_event_name(&self) -> String;

    /// Returns whether or not this event can use a running status
    ///
    /// # Return
    /// Returns whether this event can use a running status
    fn is_running_status_allowed(&self) -> bool;

    /// Gets the event type
    ///
    /// # Return
    /// Returns the event type
    fn event_type(&self) -> u8;

    /// If the running status is supported that means there is a corresponding channel.
    /// Get the channel of this event
    ///
    /// # Return
    /// Returns the channel number of this event
    fn get_channel(&self) -> u8;

    /// Gets the size of the event in bytes
    ///
    /// Returns the size of the event in bytes
    fn get_event_size(&self) -> u8;

    /// Gets the delta time represented in the midi
    ///
    /// Returns the given delta time
    fn get_delta_time(&self) -> u32;

    /// Gets the duration of the event
    ///
    /// Returns an instance of [std::time::Duration]
    fn get_time_duration(&self) -> Duration;

    /// Gets the time the event is ocurring relative to the beginning of the track
    ///
    /// Returns an instance of [std::time::Instant]
    fn get_current_time(&self) -> Instant;

    /// Converts the event into bytes that can be stored in a MIDI file
    /// without delta time
    ///
    /// Returns the bytes without delta time
    fn to_bytes(&self) -> Result<Vec<u8>, MidiError>;

    /// Converts the event into bytes that can be store in a MIDI file
    /// with delta time
    ///
    /// Returns the bytes with delta time
    fn to_bytes_delta_time(&self) -> Result<Vec<u8>, MidiError>;

    fn as_any(&self) -> &dyn Any;
}

pub trait FromBytes {
    type Output: Event;

    fn from_bytes(
        data: &[u8],
        delta_time: u32,
        time_division: TimeDivision,
        tempo: u32,
    ) -> Result<Self::Output, MidiError>;
}

pub(crate) fn dispatch_from_bytes<E>(
    data: &[u8],
    delta_time: u32,
    time_division: TimeDivision,
    tempo: u32,
) -> Result<Box<dyn Event>, MidiError>
where
    E: FromBytes + Event,
{
    E::from_bytes(data, delta_time, time_division, tempo)
        .map(|event| Box::new(event) as Box<dyn Event>)
}

#[derive(Debug)]
pub struct SerializableEvent(pub Box<dyn Event>);

#[cfg(feature = "json")]
impl Serialize for SerializableEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if let Some(note_on) = self.0.as_any().downcast_ref::<NoteOnEvent>() {
            NoteOnEvent::serialize(note_on, serializer)
        } else if let Some(note_off) = self.0.as_any().downcast_ref::<NoteOffEvent>() {
            NoteOffEvent::serialize(note_off, serializer)
        } else if let Some(channel_pressure) =
            self.0.as_any().downcast_ref::<ChannelPressureEvent>()
        {
            ChannelPressureEvent::serialize(channel_pressure, serializer)
        } else if let Some(control_change) = self.0.as_any().downcast_ref::<ControlChangeEvent>() {
            ControlChangeEvent::serialize(control_change, serializer)
        } else if let Some(pitch_bend_change) =
            self.0.as_any().downcast_ref::<PitchBendChangeEvent>()
        {
            PitchBendChangeEvent::serialize(pitch_bend_change, serializer)
        } else if let Some(polyphonic_key_pressure) =
            self.0.as_any().downcast_ref::<PolyphonicKeyPressureEvent>()
        {
            PolyphonicKeyPressureEvent::serialize(polyphonic_key_pressure, serializer)
        } else if let Some(program_change) = self.0.as_any().downcast_ref::<ProgramChangeEvent>() {
            ProgramChangeEvent::serialize(program_change, serializer)
        } else if let Some(copyright_notice) =
            self.0.as_any().downcast_ref::<CopyRightNoticeEvent>()
        {
            CopyRightNoticeEvent::serialize(copyright_notice, serializer)
        } else if let Some(cue_point) = self.0.as_any().downcast_ref::<CuePointEvent>() {
            CuePointEvent::serialize(cue_point, serializer)
        } else if let Some(end_of_track) = self.0.as_any().downcast_ref::<EndOfTrack>() {
            EndOfTrack::serialize(end_of_track, serializer)
        } else if let Some(instrument_name) = self.0.as_any().downcast_ref::<InstrumentNameEvent>()
        {
            InstrumentNameEvent::serialize(instrument_name, serializer)
        } else if let Some(key_signature) = self.0.as_any().downcast_ref::<KeySignatureEvent>() {
            KeySignatureEvent::serialize(key_signature, serializer)
        } else if let Some(lyric) = self.0.as_any().downcast_ref::<LyricEvent>() {
            LyricEvent::serialize(lyric, serializer)
        } else if let Some(marker) = self.0.as_any().downcast_ref::<MarkerEvent>() {
            MarkerEvent::serialize(marker, serializer)
        } else if let Some(midi_channel_prefix) =
            self.0.as_any().downcast_ref::<MidiChannelPrefixEvent>()
        {
            MidiChannelPrefixEvent::serialize(midi_channel_prefix, serializer)
        } else if let Some(midi_port) = self.0.as_any().downcast_ref::<MidiPortEvent>() {
            MidiPortEvent::serialize(midi_port, serializer)
        } else if let Some(sequence_number) = self.0.as_any().downcast_ref::<SequenceNumber>() {
            SequenceNumber::serialize(sequence_number, serializer)
        } else if let Some(sequencer_specific) =
            self.0.as_any().downcast_ref::<SequencerSpecificEvent>()
        {
            SequencerSpecificEvent::serialize(sequencer_specific, serializer)
        } else if let Some(set_tempo) = self.0.as_any().downcast_ref::<SetTempoEvent>() {
            SetTempoEvent::serialize(set_tempo, serializer)
        } else if let Some(smpte_offset) = self.0.as_any().downcast_ref::<SMPTEOffsetEvent>() {
            SMPTEOffsetEvent::serialize(smpte_offset, serializer)
        } else if let Some(text) = self.0.as_any().downcast_ref::<TextEvent>() {
            TextEvent::serialize(text, serializer)
        } else if let Some(time_signature) = self.0.as_any().downcast_ref::<TimeSignatureEvent>() {
            TimeSignatureEvent::serialize(time_signature, serializer)
        } else if let Some(track_name) = self.0.as_any().downcast_ref::<TrackNameEvent>() {
            TrackNameEvent::serialize(track_name, serializer)
        } else {
            unimplemented!()
        }
    }
}
