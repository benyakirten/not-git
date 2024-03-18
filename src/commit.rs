pub fn commit_command(args: &[String]) -> Result<(), anyhow::Error> {
    // 1. Get message from `-m` flag
    // 2. Get tree hash from write-tree
    // 3. Call commit-tree with the data
    // 4. Get current head for branch name
    // 5. Call update-refs with the commit hash/branch name
    Ok(())
}
