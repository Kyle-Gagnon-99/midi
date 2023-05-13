use std::time::{Duration, Instant};

#[cfg(feature = "json")]
use serde::Serialize;

use crate::{
    events::{Event, FromBytes},
    metadata::TimeDivision,
    metaevents::get_utf8_from_bytes,
    midi_error::MidiError,
};

use super::{calculate_time_duration, from_bytes_to_vlq, from_vlq_to_bytes, METAEVENT_BYTE};

const METAEVENT_BYTE_TYPE: u8 = 0x02;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "json", derive(Serialize))]
pub struct CopyRightNoticeEvent {
    pub copyright_notice: String,
    text_size: u32,
    event_size: u8,
    delta_time: u32,

    #[cfg_attr(feature = "json", serde(skip))]
    current_time: Instant,
    time_duration: Duration,
}

impl Event for CopyRightNoticeEvent {
    fn get_event_name(&self) -> String {
        String::from("Copyright Notice")
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
        let mut bytes: Vec<u8> = vec![METAEVENT_BYTE, METAEVENT_BYTE_TYPE];
        bytes.extend_from_slice(&from_vlq_to_bytes(self.text_size));
        bytes.extend_from_slice(self.copyright_notice.as_bytes());

        Ok(bytes)
    }

    fn to_bytes_delta_time(&self) -> Result<Vec<u8>, MidiError> {
        let mut bytes: Vec<u8> = from_vlq_to_bytes(self.delta_time);
        bytes.extend_from_slice(&[METAEVENT_BYTE, METAEVENT_BYTE_TYPE]);
        bytes.extend_from_slice(&from_vlq_to_bytes(self.text_size));
        bytes.extend_from_slice(self.copyright_notice.as_bytes());

        Ok(bytes)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl FromBytes for CopyRightNoticeEvent {
    type Output = Self;

    fn from_bytes(
        data: &[u8],
        delta_time: u32,
        time_division: TimeDivision,
        tempo: u32,
    ) -> Result<Self::Output, MidiError> {
        // Calculate the time duration
        let time_duration = calculate_time_duration(delta_time, time_division, tempo);

        // The data starts after the 0xFF 0x01 bytes
        let data = &data[2..];

        // The length of the data that follows is stored in VLQ
        let (data_length, num_of_bytes) = from_bytes_to_vlq(&data);

        // Ensure we are only taking the data and not anything else past that incase we are given more bytes to follow
        let data = &data[(num_of_bytes as usize)..((num_of_bytes as u32 + data_length) as usize)];

        // Convert the bytes to UTF-8 text
        let copyright_notice = get_utf8_from_bytes(&data)?;

        // The 0xFF 0x01 bytes plus the number of bytes from the VLQ size + the size of the text data
        let event_size = 2 + num_of_bytes + copyright_notice.len() as u8;

        Ok(Self {
            copyright_notice,
            text_size: data_length,
            event_size,
            delta_time,
            current_time: Instant::now(),
            time_duration,
        })
    }
}

impl CopyRightNoticeEvent {
    pub fn get_text_size(&self) -> u32 {
        self.text_size
    }
}

#[cfg(test)]
mod text_event_tests {
    use crate::{
        events::{Event, FromBytes},
        metadata::TimeDivision,
        metaevents::{copyright_notice::METAEVENT_BYTE_TYPE, from_vlq_to_bytes, METAEVENT_BYTE},
    };

    use super::CopyRightNoticeEvent;

    #[test]
    fn create_copyright_notice_struct_from_bytes_success_one_byte_vlq() {
        let test_text = String::from("(C) Kyle Gagnon");
        let test_text_len = from_vlq_to_bytes(test_text.len() as u32);

        let mut bytes: Vec<u8> = vec![METAEVENT_BYTE, METAEVENT_BYTE_TYPE];

        bytes.extend_from_slice(&test_text_len);
        bytes.extend_from_slice(test_text.as_bytes());

        let result = CopyRightNoticeEvent::from_bytes(
            &bytes,
            0,
            TimeDivision::PulsesPerQuarterNote(96),
            120,
        );
        assert!(result.is_ok());

        let text_event = result.unwrap();
        let event_size = 2 + test_text_len.len() as u8 + test_text.len() as u8;
        assert_eq!(text_event.copyright_notice, test_text);
        assert_eq!(text_event.text_size, test_text.len() as u32);
        assert_eq!(text_event.get_event_size(), event_size);
    }

    #[test]
    fn create_copyright_notice_struct_from_bytes_success_two_bytes_vlq() {
        let test_text = std::iter::repeat('A').take(128).collect::<String>();
        let test_text_len = from_vlq_to_bytes(test_text.len() as u32);

        let mut bytes: Vec<u8> = vec![METAEVENT_BYTE, METAEVENT_BYTE_TYPE];

        bytes.extend_from_slice(&test_text_len);
        bytes.extend_from_slice(test_text.as_bytes());

        let result = CopyRightNoticeEvent::from_bytes(
            &bytes,
            0,
            TimeDivision::PulsesPerQuarterNote(96),
            120,
        );
        assert!(result.is_ok());

        let text_event = result.unwrap();
        let event_size = 2 + test_text_len.len() as u8 + test_text.len() as u8;
        assert_eq!(text_event.copyright_notice, test_text);
        assert_eq!(text_event.text_size, test_text.len() as u32);
        assert_eq!(text_event.get_event_size(), event_size);
    }
}