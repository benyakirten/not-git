use std::io::{Cursor, Read};

use anyhow::Context;

const VARINT_ENCODING_BITS: u8 = 7;
// 0b1000_0000
const VARINT_CONTINUE_FLAG: u8 = 1 << VARINT_ENCODING_BITS;

const TYPE_BITS: u8 = 3;
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
    pub length: usize,
    pub data: Vec<u8>,
}

impl PackFileHeader {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, anyhow::Error> {
        let signature = String::from_utf8(bytes[0..4].to_vec())?;
        let version_number = u32::from_be_bytes(bytes[4..8].try_into()?);
        let num_objects = u32::from_be_bytes(bytes[8..12].try_into()?);

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
}

pub fn read_type_and_length(cursor: &mut Cursor<&[u8]>) -> Result<ObjectType, anyhow::Error> {
    // Using a `usize` type limits us to files that are 2^64 bytes in size.
    // I hope whatever future person is passing around files that are 16 exabytes
    // in their git repo doesn't use this code.
    let value = read_size_encoding(cursor)?;
    let object_type = left_bits(value as u8, TYPE_BITS);
    let length = value >> TYPE_BITS;

    let object_type = ObjectType::new(object_type, length)?;
    Ok(object_type)
}

fn left_bits(bits: u8, n: u8) -> u8 {
    bits >> (8 - n)
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
        // Then we add it to the value we have we have accumulated so far by using the OR operator.
        value |= (byte_value as usize) << length;

        if !more_bytes {
            return Ok(value);
        }

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
