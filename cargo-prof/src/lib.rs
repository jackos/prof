use std::process::Stdio;

use cargo_metadata::{CargoOpt, MetadataCommand};
use color_eyre::{eyre::bail, Result};

pub fn cargo_build(bin: &Option<String>) -> Result<()> {
    let mut command = std::process::Command::new("cargo");
    command.args(["build", "--release"]);

    if let Some(x) = bin {
        command.args(["--bin", &x.clone()]);
    }

    let res = command.stderr(Stdio::inherit()).output()?;
    if !res.status.success() {
        bail!(
            "Cargo could not build the project: {}",
            String::from_utf8(res.stderr)?,
        )
    }
    Ok(())
}

/// Gets the first legimate bin name from Cargo metadata
pub fn get_bin() -> Result<String> {
    let metadata = MetadataCommand::new()
        .manifest_path("./Cargo.toml")
        .features(CargoOpt::AllFeatures)
        .no_deps()
        .exec()?;

    let mut targets = Vec::new();
    for package in metadata.packages {
        targets.push(package.targets);
    }

    let targets: Vec<_> = targets
        .iter()
        .flatten()
        .filter(|x| !x.src_path.clone().into_string().contains(".cargo/registry"))
        .filter(|x| x.kind.contains(&"bin".to_string()))
        .collect();

    Ok(format!(
        "target/release/{}",
        targets.get(0).expect("no target found").name.clone()
    ))
}
