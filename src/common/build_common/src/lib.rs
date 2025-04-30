use anyhow::{Ok, Result};
use env_file_reader::read_file;
use vergen_gix::{
    BuildBuilder, CargoBuilder, Emitter, GixBuilder, RustcBuilder, SysinfoBuilder,
};

pub fn generate_envs() -> Result<()> {
    // Generate the default 'cargo:' instruction output

    // generate git info
    Emitter::default()
        .add_instructions(&BuildBuilder::all_build()?)?
        .add_instructions(&CargoBuilder::all_cargo()?)?
        .add_instructions(&GixBuilder::all_git()?)?
        .add_instructions(&RustcBuilder::all_rustc()?)?
        .add_instructions(&SysinfoBuilder::all_sysinfo()?)?
        .emit()?;

    println!("rerun-if-env-changed=NAMING_CANISTER_ENV");
    let env = if let Some(env) = option_env!("NAMING_CANISTER_ENV") {
        env
    } else {
        "dev"
    };
    println!("load env: {}", env);
    println!("warning={}", env);

    // enable feature dev_env if env is dev
    if env == "dev" {
        println!("cargo:rustc-cfg=feature=\"dev_env\"");
    }

    // load env files
    let env_parts = vec!["canister_ids", "config", "principals"];
    for env_part in env_parts {
        let env_variables = read_file(format!("../../env_configs/{}.{}.env", env, env_part))?;
        for (key, value) in env_variables {
            println!("cargo:rustc-env={}={}", key, value.replace("\n", "||||"));
        }
    }

    Ok(())
}
