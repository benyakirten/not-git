use crate::{file_hash::FileHash, ls_tree};

pub struct CheckoutConfig {
    pub branch_name: String,
}

pub fn checkout_command(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_checkout_config(args)?;

    println!("Switched to branch '{}'", config.branch_name);
    Ok(())
}

pub fn checkout_files(config: CheckoutConfig) -> Result<(), anyhow::Error> {
    if config.branch_name.starts_with("remote") {
        // TODO: Download and load the remote branch
    }
    Ok(())
}

fn parse_checkout_config(args: &[String]) -> Result<CheckoutConfig, anyhow::Error> {
    if args.len() < 1 {
        return Err(anyhow::anyhow!("Usage: checkout <branch-name>"));
    }

    Ok(CheckoutConfig {
        branch_name: args[0].clone(),
    })
}
