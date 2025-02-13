use core::option::Option;

use crate::errors::{MidiParseError, MidiParseResult};

/// The ByteReader struct allows for reading the data and moving around
///
/// ### Lifetimes
/// 'a: The lifetime of the data
pub struct ByteReader<'a> {
    /// The data to read
    data: &'a [u8],
    /// The current position in the data
    pos: u64,
    /// The length of the data
    len: u64,
}

impl<'a> ByteReader<'a> {
    /// Create a new ByteReader
    ///
    /// ### Arguments
    /// * `data` The data to read
    ///
    /// ### Returns
    /// A new ByteReader
    pub fn new(data: &'a [u8]) -> ByteReader<'a> {
        ByteReader {
            data: data,
            pos: 0,
            len: data.len() as u64,
        }
    }

    /// Read a single byte
    ///
    /// ### Returns
    /// Option<u8> The byte read, or None if the end of the data has been reached
    pub fn get_next_byte(&mut self) -> Option<u8> {
        if self.pos < self.len {
            let byte = self.data[self.pos as usize];
            self.pos += 1;
            Some(byte)
        } else {
            None
        }
    }

    /// Look at the next byte without moving the position
    ///
    /// ### Returns
    /// Option<u8> The byte at the next position, or None if the end of the data has been reached
    pub fn peek(&self) -> MidiParseResult<u8> {
        if self.pos < self.len {
            Ok(self.data[self.pos as usize])
        } else {
            Err(MidiParseError::EndOfData("End of data reached".to_string()))
        }
    }

    /// Get the length of the data
    ///
    /// ### Returns
    /// u64 The length of the data
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Get the current position in the data
    ///
    /// ### Returns
    /// u64 The current position in the data
    pub fn pos(&self) -> u64 {
        self.pos
    }

    /// Get the next N bytes
    ///
    /// ### Returns
    /// Option<[u8; N]> The next N bytes, or None if the end of the data has been reached
    pub fn get_next_bytes(&mut self, num_bytes: u64) -> MidiParseResult<Vec<u8>> {
        // Check to make sure there are enough bytes remaining
        if self.pos + num_bytes <= self.len {
            let bytes = self.data[self.pos as usize..(self.pos + num_bytes) as usize].to_vec();
            self.pos += num_bytes;
            Ok(bytes)
        } else {
            Err(MidiParseError::EndOfData(
                "Not enough bytes remaining".to_string(),
            ))
        }
    }

    /// Move the current position to a specific position
    pub fn seek(&mut self, pos: u64) {
        self.pos = pos;
    }

    /// Move the current position by a specific amount
    ///
    /// ### Arguments
    /// * `num_bytes` The number of bytes to move the position by
    pub fn move_pos(&mut self, num_bytes: i128) -> MidiParseResult<()> {
        // Verify that the new position is within the bounds of the data
        // Check for overflow or underflow
        // We are using i128 to allow for negative values but we are using a u64 for the position
        // as you can't have a negative position
        // First calculate the new position (convert to i128 to allow for negative values)
        let new_pos = self.pos as i128 + num_bytes;

        // If the new position is less than 0, report an error
        if new_pos < 0 {
            return Err(MidiParseError::InvalidDataBounds(
                "Moving position would be less than 0".to_string(),
            ));
        }

        // If the new position is greater than the length of the data, report an error
        if new_pos as u64 > self.len {
            return Err(MidiParseError::InvalidDataBounds(
                "Moving position would be greater than the length of the data".to_string(),
            ));
        }

        // If the new position is within the bounds of the data, update the position
        self.pos = new_pos as u64;
        Ok(())
    }

    /// Get the number of bytes remaining in the data
    ///
    /// ### Returns
    /// u64 The number of bytes remaining in the data
    pub fn num_bytes_remaining(&self) -> u64 {
        self.len - self.pos
    }

    /// Get the next N bytes without moving the position
    ///
    /// ### Returns
    /// Option<[u8; N]> The next N bytes, or None if the end of the data has been reached
    pub fn peek_next_bytes<const N: usize>(&self) -> Option<[u8; N]> {
        if self.pos + N as u64 <= self.len {
            let mut bytes = [0; N];
            bytes.copy_from_slice(&self.data[self.pos as usize..(self.pos + N as u64) as usize]);
            Some(bytes)
        } else {
            None
        }
    }

    /// Get the next u32 from the data
    ///
    /// ### Returns
    /// u32 The next u32 from the data
    pub fn get_next_u32(&mut self) -> MidiParseResult<u32> {
        let bytes = self.get_next_bytes(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Get the next u16 from the data
    ///
    /// ### Returns
    /// u16 The next u16 from the data
    pub fn get_next_u16(&mut self) -> MidiParseResult<u16> {
        let bytes = self.get_next_bytes(2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_next_byte() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = ByteReader::new(&data);

        assert_eq!(reader.get_next_byte(), Some(0x01));
        assert_eq!(reader.get_next_byte(), Some(0x02));
        assert_eq!(reader.get_next_byte(), Some(0x03));
        assert_eq!(reader.get_next_byte(), Some(0x04));
        assert_eq!(reader.get_next_byte(), None);
    }

    #[test]
    fn test_peek() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let reader = ByteReader::new(&data);

        assert_eq!(reader.peek(), Ok(0x01));
        assert_eq!(reader.peek(), Ok(0x01));
    }

    #[test]
    fn test_get_next_bytes() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = ByteReader::new(&data);

        assert_eq!(reader.get_next_bytes(2), Ok(vec![0x01, 0x02]));
        assert_eq!(reader.get_next_bytes(2), Ok(vec![0x03, 0x04]));
        assert!(reader.get_next_bytes(1).is_err());
    }

    #[test]
    fn test_seek() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = ByteReader::new(&data);

        reader.seek(2);
        assert_eq!(reader.get_next_byte(), Some(0x03));
    }

    #[test]
    fn test_move_pos() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = ByteReader::new(&data);

        reader.move_pos(2).unwrap();
        assert_eq!(reader.get_next_byte(), Some(0x03));
    }

    #[test]
    fn test_num_bytes_remaining() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = ByteReader::new(&data);

        reader.move_pos(2).unwrap();
        assert_eq!(reader.num_bytes_remaining(), 2);
    }

    #[test]
    fn test_peek_next_bytes() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let reader = ByteReader::new(&data);

        assert_eq!(reader.peek_next_bytes::<2>(), Some([0x01, 0x02]));
        assert_eq!(reader.peek_next_bytes::<2>(), Some([0x01, 0x02]));
    }

    #[test]
    fn test_move_pos_error() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = ByteReader::new(&data);

        assert!(reader.move_pos(-1).is_err());
        assert!(reader.move_pos(5).is_err());

        reader.move_pos(2).unwrap();
        assert!(reader.move_pos(-3).is_err());
        assert!(reader.move_pos(1).is_ok());
        assert!(reader.move_pos(1000).is_err());
    }

    #[test]
    fn test_get_next_u32() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut reader = ByteReader::new(&data);

        assert_eq!(reader.get_next_u32(), Ok(0x01020304));
    }

    #[test]
    fn test_get_next_u16() {
        let data = [0x01, 0x02];
        let mut reader = ByteReader::new(&data);

        assert_eq!(reader.get_next_u16(), Ok(0x0102));
    }
}
