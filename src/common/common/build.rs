use anyhow::{Ok, Result};
use env_file_reader::read_file;

fn main() -> Result<()> {
    // Generate the default 'cargo:' instruction output
    let env = if let Some(env) = option_env!("NAMING_CANISTER_ENV") {
        env
    } else {
        "dev"
    };
    println!("load env: {}", env);
    println!("rerun-if-env-changed=NAMING_CANISTER_ENV");
    let env_parts = vec!["canister_ids", "config", "principals"];
    for env_part in env_parts {
        let env_variables = read_file(format!("../../env_configs/{}.{}.env", env, env_part))?;
        for (key, value) in env_variables {
            println!("cargo:rustc-env={}={}", key, value.replace("\n", "||||"));
        }
    }

    Ok(())
}
