use std::{io::Cursor, path::PathBuf};

use bytes::Bytes;

use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;

use crate::{
    file_hash::FileHash,
    hash_object,
    ls_tree::FileType,
    packfile::{self, ObjectEntry, PackFileHeader},
};

pub struct CloneConfig {
    pub url: String,
    pub path: PathBuf,
}

#[derive(Debug)]
struct GitRef {
    mode: String,
    commit_hash: FileHash,
    branch: String,
    is_head: bool,
}

pub fn clone_command(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_clone_config(args)?;
    clone(config)?;
    Ok(())
}

pub fn clone(config: CloneConfig) -> Result<Vec<ObjectEntry>, anyhow::Error> {
    // https://www.git-scm.com/docs/http-protocol
    // We could use async functions or we could run this as single-threaded with blocking calls
    // We will use blocking calls for simplicity/ease of use. I don't think there's a part that
    // would benefit from async calls yet.
    let client = Client::new();
    let refs = discover_references(
        &client,
        &format!("{}/info/refs", config.url),
        "git-upload-pack",
    )?;

    // TODO: Write all other refs into the `.git/packed-refs` file.

    let head_ref = refs
        .iter()
        .find(|r| r.is_head)
        .ok_or_else(|| anyhow::anyhow!("No HEAD ref found"))?;

    let commit = get_commit(&client, &config.url, &head_ref.commit_hash)?;

    // Header will contain the following:
    // 1. 0008NAK\n - because we have ended negotiation
    // 2. 4-byte signature which can be stringified to PACK
    // 3. 4-byte version number - always 2 or 3
    // 4. 4-byte number of objects
    let header = PackFileHeader::from_bytes(commit[..20].to_vec())?;

    let mut objects: Vec<ObjectEntry> = vec![];
    let mut cursor = Cursor::new(&commit[20..]);

    for _ in 0..header.num_objects {
        let position = cursor.position() as usize;
        let object_type = packfile::read_type_and_length(&mut cursor)?;

        let object = match object_type {
            packfile::ObjectType::Commit(size) => {
                let (data, file_hash) =
                    packfile::decode_undeltified_data(FileType::Commit, &mut cursor)?;
                ObjectEntry {
                    object_type,
                    size,
                    data,
                    position,
                    file_hash,
                }
            }
            packfile::ObjectType::Tree(size) => {
                let (data, file_hash) =
                    packfile::decode_undeltified_data(FileType::Tree, &mut cursor)?;
                ObjectEntry {
                    object_type,
                    size,
                    data,
                    position,
                    file_hash,
                }
            }
            packfile::ObjectType::Blob(size) => {
                let (data, file_hash) =
                    packfile::decode_undeltified_data(FileType::Blob, &mut cursor)?;
                ObjectEntry {
                    object_type,
                    size,
                    data,
                    position,
                    file_hash,
                }
            }
            packfile::ObjectType::Tag(size) => {
                let (data, file_hash) =
                    packfile::decode_undeltified_data(FileType::Tag, &mut cursor)?;
                ObjectEntry {
                    object_type,
                    size,
                    data,
                    position,
                    file_hash,
                }
            }
            packfile::ObjectType::OfsDelta(size) => {
                todo!()
                // let data = packfile::read_obj_offset_data(&mut cursor)?;
                // ObjectEntry {
                //     object_type,
                //     size,
                //     data,
                //     position,
                // }
            }
            packfile::ObjectType::RefDelta(size) => {
                let data = packfile::read_obj_ref_data(objects, &mut cursor)?;
                todo!()
            }
        };

        // Though we need to look up values by an exact value, until we get to 50k+ objects,
        // a linear search using a vector is more efficient than a hash table since the items are stored
        // in a contiguous block of memory. That said, if we used a hash map, we should either
        // need to look up values by position or by their hash.
        // We could use two different hash maps, but I doubt the memory cost is worth it.
        objects.push(object);
    }

    Ok(objects)
}

fn get_commit(client: &Client, url: &str, commit_hash: &FileHash) -> Result<Bytes, anyhow::Error> {
    let request_url = format!("{}/git-upload-pack", url);
    // 0000 is the termination code
    // 0009done is added to indicate that this is the final request in negotiation
    let body = format!("0032want {}\n00000009done\n", commit_hash.full_hash());

    let resp = client
        .post(request_url)
        .body(body)
        .header(CONTENT_TYPE, "application/x-git-upload-pack-request")
        .send()?;

    let status = resp.status().as_u16();
    if status != 200 && status != 304 {
        return Err(anyhow::anyhow!(format!(
            "Failed to get commit: Status code must be either 200 or 304, received {}",
            status
        )));
    }

    let want_content_type = "application/x-git-upload-pack-result";
    let headers = resp.headers();
    let content_type = headers
        .get(CONTENT_TYPE)
        .ok_or_else(|| anyhow::anyhow!("Content-Type must equal {}", want_content_type))?
        .to_str()?;

    if content_type != want_content_type {
        return Err(anyhow::anyhow!(
            "Content-Type must equal {}, received {}",
            want_content_type,
            content_type
        ));
    }

    // BEWARE: DO NOT CONVERT TO STRING
    // Some of the response can be encoded to string
    // but some of it can't. Calling `.text` is like calling
    // .from_utf8_lossy - bytes that cannot be decoded to string
    // are replaced with a special unicode character
    let bytes = resp.bytes()?;
    Ok(bytes)
}

fn discover_references(
    client: &Client,
    url: &str,
    service_name: &str,
) -> Result<Vec<GitRef>, anyhow::Error> {
    let request_url = format!("{}?service={}", url, service_name);
    let resp = client.get(request_url).send()?;

    let status = resp.status().as_u16();
    if status != 200 && status != 304 {
        return Err(anyhow::anyhow!(format!(
            "Failed to get refs: Status code must be either 200 or 304, received {}",
            status
        )));
    }

    let want_content_type = format!("application/x-{}-advertisement", service_name);
    let headers = resp.headers();
    let content_type = headers
        .get(CONTENT_TYPE)
        .ok_or_else(|| anyhow::anyhow!(format!("Content-Type must equal {}", want_content_type)))?
        .to_str()?;

    if content_type != want_content_type {
        return Err(anyhow::anyhow!(format!(
            "Content-Type must equal {}, received {}",
            want_content_type, content_type
        )));
    }

    let text = resp.text()?;
    let first_chars_err = Err(anyhow::anyhow!(
        "Invalid packet format: first five bytes must match the pattern [0-9a-z]#"
    ));

    let mut first_chars = text[..5].chars();
    for _ in 0..4 {
        let character = first_chars.next();
        if character.is_none() {
            return first_chars_err;
        }

        let character = character.unwrap();

        if !character.is_numeric() && !character.is_ascii_lowercase() {
            return first_chars_err;
        }
    }

    let pound_char = first_chars.next();
    if pound_char != Some('#') {
        return first_chars_err;
    }

    if !text.ends_with("0000") {
        return Err(anyhow::anyhow!(
            "Invalid packet format: last four bytes must be 0000"
        ));
    }

    let mut lines = text.lines();
    let first_line = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("No lines in packet"))?;

    if !first_line.ends_with("service=git-upload-pack") {
        return Err(anyhow::anyhow!(
            "Invalid packet format: first line must end with service=git-upload-pack"
        ));
    }

    // Head line should read 00000153{HEAD_SHA} HEAD\0{capabilities}
    // Since we are interested only in the HEAD_SHA, we cut away everything else
    let head_line = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("No HEAD line in packet"))?;

    let head_ref = FileHash::from_sha(head_line[8..48].to_string())?;

    let refs: Vec<GitRef> = lines
        .filter_map(|line| {
            if line == "0000" {
                return None;
            }
            let parts = line.split_once(" ");
            if parts.is_none() {
                return None;
            }

            let (mode_and_hash, branch) = parts.unwrap();

            let mode = mode_and_hash[0..4].to_string();
            let commit_hash = mode_and_hash[4..].to_string();
            let commit_hash = match FileHash::from_sha(commit_hash) {
                Ok(hash) => hash,
                Err(_) => return None,
            };

            let is_head = commit_hash.full_hash() == head_ref.full_hash();
            let git_ref = GitRef {
                mode,
                commit_hash,
                branch: branch.to_string(),
                is_head,
            };
            Some(git_ref)
        })
        .collect();

    Ok(refs)
}

fn parse_clone_config(args: &[String]) -> Result<CloneConfig, anyhow::Error> {
    if args.len() == 0 || args.len() > 2 {
        return Err(anyhow::anyhow!("Usage: clone <url> [<path>]"));
    }

    let url = args[0].to_string();
    let path = if args.len() == 2 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from(".")
    };

    Ok(CloneConfig { url, path })
}

fn parse_objects(num_object: u32) -> Result<(), anyhow::Error> {
    todo!()
}
