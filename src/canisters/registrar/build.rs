use anyhow::{Ok, Result};
use build_common::generate_envs;

fn main() -> Result<()> {
    generate_envs()?;
    Ok(())
}
