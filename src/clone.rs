use reqwest::blocking::Client;
use std::path::PathBuf;

use crate::file_hash::FileHash;

pub struct CloneConfig {
    pub url: String,
    pub path: PathBuf,
}

#[derive(Debug)]
struct GitRef {
    mode: String,
    commit_hash: FileHash,
    branch: String,
}

pub fn clone_command(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_clone_config(args)?;
    clone(config)?;
    Ok(())
}

pub fn clone(config: CloneConfig) -> Result<(), anyhow::Error> {
    // https://www.git-scm.com/docs/http-protocol
    let client = Client::new();
    let refs = discover_references(
        client,
        &format!("{}/info/refs", config.url),
        "git-upload-pack",
    )?;

    println!("Cloning into {:?}", config.path);
    for git_ref in refs {
        println!("{:?}", git_ref);
    }
    Ok(())
}

fn discover_references(
    client: Client,
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

    let want_header = format!("application/x-{}-advertisement", service_name);
    let headers = resp.headers();
    let content_type = headers
        .get(reqwest::header::CONTENT_TYPE)
        .ok_or_else(|| anyhow::anyhow!(format!("Content-Type must equal {}", want_header)))?
        .to_str()?;

    if content_type != want_header {
        return Err(anyhow::anyhow!(format!(
            "Content-Type must equal {}, received {}",
            want_header, content_type
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

    let refs: Vec<GitRef> = lines
        .skip(1)
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

            let git_ref = GitRef {
                mode,
                commit_hash,
                branch: branch.to_string(),
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
