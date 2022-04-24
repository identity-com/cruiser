#![warn(unused_import_braces, unused_imports, clippy::pedantic)]

use crates_io_api::AsyncClient;
use std::error::Error;
use std::process::exit;
use std::time::Duration;
use structopt::StructOpt;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use toml::Value;

#[derive(StructOpt)]
#[structopt(
    name = "check_versions",
    about = "Checks if the given cargo.toml versions are already on crates.io"
)]
struct Opt {
    #[structopt(short, long)]
    file: Vec<String>,
}

#[allow(clippy::too_many_lines)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();
    let client = AsyncClient::new("cruiser-ci (buzzec@buzzec.net)", Duration::from_secs(1))?;
    for file in opt.file {
        let mut binary_file = File::open(&file).await?;

        let metadata = binary_file.metadata().await?;
        if !metadata.is_file() {
            panic!("{} is not a file", file);
        }
        let mut buffer = Vec::new();
        while binary_file.read_buf(&mut buffer).await? > 0 {}
        let cargo_toml = String::from_utf8(buffer)?.parse::<Value>()?;
        let table = cargo_toml.as_table().ok_or("cargo.toml is not a table")?;
        let package = table
            .get("package")
            .ok_or("cargo.toml does not have a package section")?
            .as_table()
            .ok_or("package section is not a table")?;
        let name = package
            .get("name")
            .ok_or("package section does not have a name")?
            .as_str()
            .ok_or("name is not a string")?;
        let version = package
            .get("version")
            .ok_or("package section does not have a version")?
            .as_str()
            .ok_or("version is not a string")?;

        let krate = client.get_crate(&name).await?;
        if krate.versions.into_iter().any(|v| v.num == version) {
            eprintln!("{} version {} is already on crates.io", name, version);
            exit(104);
        }
    }

    Ok(())
}
