use std::io::{Cursor, ErrorKind, Read};

use anyhow::Context;
use flate2::bufread::ZlibDecoder;

use crate::{file_hash::FileHash, hash_object, ls_tree::FileType};

const VARINT_ENCODING_BITS: u8 = 7;

// 0b1000_0000 - (byte) & MSB_MASK returns the bit value of the first bit
const MSB_MASK: u8 = 1 << VARINT_ENCODING_BITS;

const TYPE_BITS: u8 = 3;
// 7 - 3 = 4 bits
const TYPE_BYTE_SIZE_BITS: u8 = VARINT_ENCODING_BITS - TYPE_BITS;

// Maximum number of bytes that might be specified in offset.
const MAX_OFFSET_BYTES: usize = 4;

// Maximum number of mgihts that be specified in size.
const MAX_SIZE_BYTES: usize = 3;

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
    pub position: usize,
    pub file_hash: FileHash,
}

pub enum DeltaInstruction {
    Copy(CopyInstruction),
    Insert(InsertInstruction),
    End,
}

pub struct CopyInstruction {
    // Offset of the first byte to copy.
    pub offset: usize,
    // Number of bytes to copy.
    pub size: usize,
}

pub struct InsertInstruction {
    // Number of bytes to copy from delta data to the target data
    pub size: u8,
}

impl PackFileHeader {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, anyhow::Error> {
        let head = String::from_utf8(bytes[0..8].to_vec())?;

        if head != "0008NAK\n" {
            return Err(anyhow::anyhow!("Invalid packfile header"));
        }

        let signature = String::from_utf8(bytes[8..12].to_vec())?;
        let version_number = u32::from_be_bytes(bytes[12..16].try_into()?);
        let num_objects = u32::from_be_bytes(bytes[16..].try_into()?);

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
}

pub fn decode_undeltified_data(
    file_type: FileType,
    cursor: &mut Cursor<&[u8]>,
) -> Result<(Vec<u8>, FileHash), anyhow::Error> {
    let data = read_next_zlib_data(cursor)?;
    let hash = hash_object::hash_and_write_object(&file_type, &mut data.clone())?;
    Ok((data, hash))
}

pub fn read_obj_offset_data(
    objects: &Vec<ObjectEntry>,
    cursor: &mut Cursor<&[u8]>,
) -> Result<(Vec<u8>, FileHash), anyhow::Error> {
    // We need to read the offset from the packfile. The offset is variable-sized
    // but negative so we are guaranteed to have seen it before now.
    // Then we need to find the object that starts at that position.
    // Once we have that file, then we can get the data delta.
    let current_position = cursor.position();

    // This will move the cursor ahead by the number of bytes in the varint
    let negative_offset = read_varint_bytes(cursor)?;
    let position = current_position - negative_offset as u64;
    let object = objects
        .iter()
        .find(|object| object.position as u64 == position);

    if object.is_none() {
        return Err(anyhow::anyhow!(format!(
            "Unable to find object with position {} in packfile",
            position
        )));
    }

    compile_file_from_deltas(cursor, object.unwrap())
}

pub fn read_obj_ref_data(
    objects: &Vec<ObjectEntry>,
    cursor: &mut Cursor<&[u8]>,
) -> Result<(Vec<u8>, FileHash), anyhow::Error> {
    let mut ref_sha: [u8; 20] = [0; 20];
    cursor.read_exact(&mut ref_sha)?;

    let hash = hex::encode(ref_sha);
    let hash = FileHash::from_sha(hash)?;

    let object = objects
        .iter()
        .find(|object| object.file_hash.full_hash() == hash.full_hash());

    if object.is_none() {
        return Err(anyhow::anyhow!(format!(
            "Unable to find object with hash {} in packfile",
            hash.full_hash()
        )));
    }

    compile_file_from_deltas(cursor, object.unwrap())
}

fn compile_file_from_deltas(
    cursor: &mut Cursor<&[u8]>,
    object: &ObjectEntry,
) -> Result<(Vec<u8>, FileHash), anyhow::Error> {
    let delta_data = read_next_zlib_data(cursor)?;
    let file_contents = apply_deltas(&object, delta_data)?;
    let file_type = match get_object_file_type(&object.object_type) {
        Ok(file_type) => file_type,
        Err(e) => {
            println!("Unable to identify file type, defaulting to blob: {}", e);
            FileType::Blob
        }
    };

    let hash = hash_object::hash_and_write_object(&file_type, &mut file_contents.clone())?;

    Ok((file_contents, hash))
}

fn apply_deltas(target: &ObjectEntry, delta_data: Vec<u8>) -> Result<Vec<u8>, anyhow::Error> {
    let mut cursor = Cursor::new(delta_data.as_slice());

    let source_length = read_varint_bytes(&mut cursor)?;
    let final_length = read_varint_bytes(&mut cursor)?;

    let mut data = Vec::with_capacity(final_length);

    if source_length != target.size {
        // TODO: Log inconsistent sizes
    }

    loop {
        let instruction = read_instruction(&mut cursor)?;
        let new_data = match instruction {
            DeltaInstruction::Insert(instruction) => {
                apply_insert_instruction(&mut cursor, instruction.size as usize)?
            }
            DeltaInstruction::Copy(instruction) => {
                apply_copy_instruction(target, instruction.offset, instruction.size)?
            }
            DeltaInstruction::End => break,
        };

        data.extend(new_data);
    }

    Ok(data)
}

fn read_instruction(cursor: &mut Cursor<&[u8]>) -> Result<DeltaInstruction, anyhow::Error> {
    let mut byte = match read_byte(cursor) {
        Ok(byte) => byte,
        // We have finihed reading instructions when we get to the EOF
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(DeltaInstruction::End),
        Err(e) => return Err(e.into()),
    };

    // Test if the most significant bit (first) is 0 or 1 by masking all bits except the first
    // 0 - insertion
    // 1 - copy
    let instruction: DeltaInstruction = if byte & MSB_MASK == 0 {
        // Get the last 7 bits of the byte
        let size = byte & !MSB_MASK;
        let instruction = InsertInstruction { size };
        DeltaInstruction::Insert(instruction)
    } else {
        let offset = get_copy_instruction_data(cursor, MAX_OFFSET_BYTES as u8, &mut byte)?;
        let size = match get_copy_instruction_data(cursor, MAX_SIZE_BYTES as u8, &mut byte)? {
            // Per the git instructions, if a size of 0 is specified
            // it should be interpreted as 0x10000 == 2^16 == 65536.
            0 => 0x10000,
            size => size,
        };

        let instruction = CopyInstruction { offset, size };
        DeltaInstruction::Copy(instruction)
    };

    Ok(instruction)
}

/// Get a offset or size for a copy instruction.
/// CF https://github.com/git/git/blob/795ea8776befc95ea2becd8020c7a284677b4161/Documentation/gitformat-pack.txt#L128
/// In the original byte to get the instruction, 0b1xxx_xxxx - starting with little endian order (i.e. from the right)
/// the first 4 bits show how many bytes are in the offset, and idem for the last 3 in relation to the size.
/// The sum of the 1s in each section of the instructions indicate how much should be read, and their positioning
/// indicates in which order the bytes should be inserted in a u32.
/// If we have the copy instruction 0b1101_0101, we have an offset of `0101` and a size of `101`.
/// This indicates that the offset will come from two bytes, which will represent byte 2 and byte 4.
/// If the bytes are read as 0b0010_010 0b1001_0010 then the final value will be
/// 0b0000_0000 0b0010_0101 0b0000_0000 0b1001_0010
/// Same for the size but with 3 bytes (so maximum value is 16mb) - also only 3 bytes will be read
fn get_copy_instruction_data(
    cursor: &mut Cursor<&[u8]>,
    num_bytes: u8,
    instruction_bits: &mut u8,
) -> Result<usize, anyhow::Error> {
    let mut value = 0;

    for index in 0..num_bytes {
        // If the last bit is 1 then read the byte
        if *instruction_bits & 1 == 1 {
            let byte = read_byte(cursor)?;

            // Move the read byte over by 8 bits incrementally
            // because we're reading this in little-endian order
            // e.g. 0b1101_1010 will be read as 0b1011_0110 0b0000_0000
            // Then we add the the bits to the value
            value |= (byte as usize) << (index * 8);
        }

        // Move over the instruction bits by 1
        *instruction_bits >>= 1;
    }

    Ok(value)
}

fn apply_copy_instruction(
    target: &ObjectEntry,
    offset: usize,
    size: usize,
) -> Result<Vec<u8>, anyhow::Error> {
    let data = target.data.get(offset..offset + size).ok_or_else(|| {
        anyhow::anyhow!(format!(
            "Unable to get data from offset {} to {} in target data",
            offset,
            offset + size
        ))
    })?;

    Ok(data.to_vec())
}

fn apply_insert_instruction(
    cursor: &mut Cursor<&[u8]>,
    size: usize,
) -> Result<Vec<u8>, anyhow::Error> {
    let mut data = vec![0; size];
    cursor.read_exact(&mut data)?;

    Ok(data)
}

fn read_next_zlib_data(mut cursor: &mut Cursor<&[u8]>) -> Result<Vec<u8>, anyhow::Error> {
    let starting_position = cursor.position();
    let mut data = vec![];

    // We don't know the size of the compressed blob, so we just read until it gives up.
    let mut decoder = ZlibDecoder::new(&mut cursor);
    decoder.read_to_end(&mut data)?;

    // Since we don't know the size of the compressed blob before we read it,
    // we need to manually move the cursor to the correct position after.
    let read_bytes = decoder.total_in();
    cursor.set_position(starting_position + read_bytes);

    Ok(data)
}

/// We need to read the packfile in little-endian order. The first three bits are the object type.
pub fn read_type_and_length(cursor: &mut Cursor<&[u8]>) -> Result<ObjectType, anyhow::Error> {
    // Using a `usize` type limits us to files that are 2^61 bytes in size.
    // I hope whatever future person is passing around files that are 2 exabytes
    // in their git repo doesn't use this code.
    let value = read_varint_bytes(cursor)?;

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

/// We receive a variable-sized encoded value from the packfile. We want to get all the bytes
/// that represent the type and length of the object. Using a cursor allows us to advance
/// that distance without keeping track of the current position in the buffer.
pub fn read_varint_bytes(packfile_reader: &mut Cursor<&[u8]>) -> Result<usize, anyhow::Error> {
    let mut value = 0;
    let mut length = 0;

    loop {
        let (byte_value, more_bytes) = read_varint_byte(packfile_reader)?;
        // We append the next 7 bits in little-endian order (add the new bits on the right side).
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

    // !MSB_MASK is the same as 0b0111_1111
    // We use it to block the first bit of the byte (the continuation flag)
    // because it's 0 so no matter what, it & a binary will be 0
    // Therefore we get the value of last 7 bits from the byte.
    let value = byte & !MSB_MASK;

    // We check if the continuation flag is 0 or 1.
    // b1000_0000 (MSB_MASK) & b1xxx_xxxx will equal 1
    // MSB_MASK & b0xxx_xxxx will equal 0
    let more_bytes = byte & MSB_MASK != 0;

    Ok((value, more_bytes))
}

pub fn read_byte(cursor: &mut Cursor<&[u8]>) -> Result<u8, std::io::Error> {
    let mut bytes: [u8; 1] = [0];
    cursor.read_exact(&mut bytes)?;
    Ok(bytes[0])
}

fn get_object_file_type(object_type: &ObjectType) -> Result<FileType, anyhow::Error> {
    match object_type {
        ObjectType::Commit(_) => Ok(FileType::Commit),
        ObjectType::Tree(_) => Ok(FileType::Tree),
        ObjectType::Blob(_) => Ok(FileType::Blob),
        ObjectType::Tag(_) => Ok(FileType::Tag),
        // Can a ref/ofs object point to another ref/ofs object?
        // TODO: Encode that information in the object so we can
        // recursively find the original reference
        ObjectType::OfsDelta(_) => Err(anyhow::anyhow!("Invalid object type: OfsDelta")),
        ObjectType::RefDelta(_) => Err(anyhow::anyhow!("Invalid object type: RefDelta")),
    }
}
