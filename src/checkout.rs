use crate::{file_hash::FileHash, ls_tree};

pub struct CheckoutConfig {
    pub branch_name: String,
}

pub fn checkout_command(args: &[String]) -> Result<(), anyhow::Error> {
    let config = parse_checkout_config(args)?;

    println!("Switched to branch '{}'", config.branch_name);
    Ok(())
}

pub fn load_commit(config: CheckoutConfig) -> Result<(), anyhow::Error> {
    // let tree = ls_tree::
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
