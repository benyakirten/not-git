use std::io::{Cursor, Read};

use anyhow::Context;
use flate2::bufread::ZlibDecoder;

const VARINT_ENCODING_BITS: u8 = 7;
// 0b1000_0000
const VARINT_CONTINUE_FLAG: u8 = 1 << VARINT_ENCODING_BITS;

const TYPE_BITS: u8 = 3;
// 7 - 3 = 4 bits
const TYPE_BYTE_SIZE_BITS: u8 = VARINT_ENCODING_BITS - TYPE_BITS;

#[derive(Debug)]
pub struct PackFileHeader {
    pub signature: String,
    pub version_number: u32,
    pub num_objects: u32,
}

#[derive(Debug)]
pub struct ObjectEntry {
    pub object_type: ObjectType,
    pub size: usize,
    pub data: Vec<u8>,
}

impl PackFileHeader {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, anyhow::Error> {
        let signature = String::from_utf8(bytes[0..4].to_vec())?;
        let version_number = u32::from_be_bytes(bytes[4..8].try_into()?);
        let num_objects = u32::from_be_bytes(bytes[8..].try_into()?);

        if signature != "PACK" {
            return Err(anyhow::anyhow!("Invalid packfile signature"));
        }

        if version_number != 2 && version_number != 3 {
            return Err(anyhow::anyhow!("Invalid packfile version number"));
        }

        Ok(PackFileHeader {
            signature,
            version_number,
            num_objects,
        })
    }
}

#[derive(Debug)]
pub enum ObjectType {
    Commit(usize),
    Tree(usize),
    Blob(usize),
    Tag(usize),
    OfsDelta(usize),
    RefDelta(usize),
}

impl ObjectType {
    pub fn new(object_type: u8, size: usize) -> Result<Self, anyhow::Error> {
        match object_type {
            1 => Ok(ObjectType::Commit(size)),
            2 => Ok(ObjectType::Tree(size)),
            3 => Ok(ObjectType::Blob(size)),
            4 => Ok(ObjectType::Tag(size)),
            6 => Ok(ObjectType::OfsDelta(size)),
            7 => Ok(ObjectType::RefDelta(size)),
            _ => Err(anyhow::anyhow!("Invalid object type")),
        }
    }

    pub fn name(&self) -> String {
        match self {
            ObjectType::Commit(_) => "commit".to_string(),
            ObjectType::Tree(_) => "tree".to_string(),
            ObjectType::Blob(_) => "blob".to_string(),
            ObjectType::Tag(_) => "tag".to_string(),
            ObjectType::OfsDelta(_) => "ofs-delta".to_string(),
            ObjectType::RefDelta(_) => "ref-delta".to_string(),
        }
    }

    pub fn length(&self) -> usize {
        match self {
            ObjectType::Commit(length) => *length,
            ObjectType::Tree(length) => *length,
            ObjectType::Blob(length) => *length,
            ObjectType::Tag(length) => *length,
            ObjectType::OfsDelta(length) => *length,
            ObjectType::RefDelta(length) => *length,
        }
    }

    pub fn parse_data(&self, cursor: Cursor<&[u8]>) -> Result<(), anyhow::Error> {
        match self {
            ObjectType::Commit(size) => ObjectType::decode_undeltified_data(size, cursor),
            ObjectType::Tree(size) => ObjectType::decode_undeltified_data(size, cursor),
            ObjectType::Blob(size) => ObjectType::decode_undeltified_data(size, cursor),
            ObjectType::Tag(size) => ObjectType::decode_undeltified_data(size, cursor),
            ObjectType::OfsDelta(size) => ObjectType::read_obj_offset_data(size, cursor),
            ObjectType::RefDelta(size) => ObjectType::read_obj_ref_data(size, cursor),
        }
    }

    fn decode_undeltified_data(
        size: &usize,
        mut cursor: Cursor<&[u8]>,
    ) -> Result<(), anyhow::Error> {
        let mut data = vec![0; *size];

        let mut decoder = ZlibDecoder::new(&mut cursor);
        decoder.read_exact(&mut data)?;

        println!("{:?}", data);
        Ok(())
    }

    fn read_obj_offset_data(size: &usize, mut cursor: Cursor<&[u8]>) -> Result<(), anyhow::Error> {
        todo!()
    }

    fn read_obj_ref_data(size: &usize, mut cursor: Cursor<&[u8]>) -> Result<(), anyhow::Error> {
        todo!()
    }
}

pub fn read_type_and_length(cursor: &mut Cursor<&[u8]>) -> Result<ObjectType, anyhow::Error> {
    // Using a `usize` type limits us to files that are 2^61 bytes in size.
    // I hope whatever future person is passing around files that are 2 exabytes
    // in their git repo doesn't use this code.
    let value = read_size_encoding(cursor)?;

    let object_type = get_object_type_bits(value) as u8;
    let size = get_object_size(value);

    let object_type = ObjectType::new(object_type, size)?;
    Ok(object_type)
}

/// Given the packfile encoding, the last seven bits are three bits for the
/// object type and four bits for the size. We want to remove the three bits
/// for the object type then get an integer representing the size from
/// the total size - 7 bits + last 4 bits.
fn get_object_size(value: usize) -> usize {
    // Given the entire value, we want to get the last 4 bits
    // e.g. 0b0100_1000_1011 becomes 0b1011
    let final_four_bits = keep_bits(value, TYPE_BYTE_SIZE_BITS);

    // Remove the final 7 bits from the file
    // e.g. 0b0100_1000_1011 becomes 0b0100_1
    let value_with_no_final_bits = value >> VARINT_ENCODING_BITS;

    // Add three empty bits to the end
    // e.g. 0b0100_1000_1011 -> 0b0100_1 -> 0b0100_1000_0000
    let value_with_no_final_bits = value_with_no_final_bits << TYPE_BYTE_SIZE_BITS;

    // Fill the final 4 bits in with the value from the first step
    // e.g. 0b0100_1000_0000 | 0b1011 = 0b0100_1000_1011
    return value_with_no_final_bits | final_four_bits;
}

fn get_object_type_bits(value: usize) -> usize {
    // The value will be encoded as groups of concatenated 7 bits representing the size
    // Except the last 7 bits includes the object type in the first three bits
    // We will have something like `0b1010001000101010`, and we need to shift this over
    // by 4 bits then take the next 3 bits - 4 in this case, the object type.

    // Shift right by 4 bits - get rid of the size bits in the last 7 bits
    let size_and_object_type = value >> TYPE_BYTE_SIZE_BITS;

    // Read the last 3 bits
    return keep_bits(size_and_object_type, TYPE_BITS);
}

fn keep_bits(value: usize, bits: u8) -> usize {
    // Shift 1 by however many bits we want to keep
    // e.g. with size 3: 0b1000
    let mask = 1 << bits;

    // Subtract 1 so that all the bits the right of it are 1
    // e.g. 0b1000 - 1 = 0b0111
    let mask = mask - 1;

    // Only retain the bits that are in the mask - 0 (mask) & 1 (value) = 0
    // Therefore only the bits that have a 1 in the mask will be kept
    // The mask will be applied to the rightmost bits
    value & mask
}

// We receive a variable-sized encoded value from the packfile. We want to get all the bytes
// that represent the type and length of the object. Using a cursor allows us to advance
// that distance without keeping track of the current position in the buffer.
pub fn read_size_encoding(packfile_reader: &mut Cursor<&[u8]>) -> Result<usize, anyhow::Error> {
    let mut value = 0;
    let mut length = 0;

    loop {
        let (byte_value, more_bytes) = read_varint_byte(packfile_reader)?;
        // We shift the byte value to the left by 7 * the number of reps we've done so far
        // Then we add it to the front of the current value.
        // If we get 0b0001000 in the first 7 bits then 0b0101010 in the next 7 bits
        // Then we should get 0b0101010_0001000 - note these are group of 7 bits
        // The leftmost bits should be from the 7 rightmost bits of the most recently read byte
        value |= (byte_value as usize) << length;
        if !more_bytes {
            return Ok(value);
        }

        // Increment how much we left shift by 7 bits.
        length += VARINT_ENCODING_BITS;
    }
}

// We read a single byte from the cursor. We divide it into two parts: the 7-bit value and
// the flag for whether there are more bytes to read.
pub fn read_varint_byte(packfile_reader: &mut Cursor<&[u8]>) -> Result<(u8, bool), anyhow::Error> {
    let mut bytes: [u8; 1] = [0];

    packfile_reader
        .read_exact(&mut bytes)
        .context("Unable to read more bytes from response but no end flag has been received")?;

    let [byte] = bytes;

    // !VARINT_CONTINUE_FLAG is the same as 0b0111_1111
    // We use it to block the first bit of the byte (the continuation flag)
    // because it's 0 so no matter what, it & a binary will be 0
    // Therefore we get the value of last 7 bits from the byte.
    let value = byte & !VARINT_CONTINUE_FLAG;

    // We check if the continuation flag is 0 or 1.
    // b1000_0000 (VARINT_CONTINUE_FLAG) & b1xxx_xxxx will equal 1
    // VARINT_CONTINUE_FLAG & b0xxx_xxxx will equal 0
    let more_bytes = byte & VARINT_CONTINUE_FLAG != 0;

    Ok((value, more_bytes))
}
