use reqwest::blocking::Client;
use std::path::PathBuf;

pub struct CloneConfig {
    pub url: String,
    pub path: PathBuf,
}

pub fn clone_command(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_clone_config(args)?;
    clone(config)?;
    Ok(())
}

pub fn clone(config: CloneConfig) -> Result<(), anyhow::Error> {
    // https://www.git-scm.com/docs/http-protocol
    let client = Client::new();
    let lines = discover_references(
        client,
        &format!("{}/info/refs", config.url),
        "git-upload-pack",
    )?;

    Ok(())
}

fn discover_references(
    client: Client,
    url: &str,
    service_name: &str,
) -> Result<Vec<String>, anyhow::Error> {
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

    let lines: Vec<String> = text.lines().map(|l| l.to_string()).collect();
    if !lines[0].ends_with("service=git-upload-pack") {
        return Err(anyhow::anyhow!(
            "Invalid packet format: first line must end with service=git-upload-pack"
        ));
    }

    Ok(lines)
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
