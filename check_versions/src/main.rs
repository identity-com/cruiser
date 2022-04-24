#![warn(unused_import_braces, unused_imports, clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::cast_possible_truncation)]

use crates_io_api::AsyncClient;
use futures::future::try_join_all;
use std::error::Error;
use std::os::unix::fs::MetadataExt;
use std::str::FromStr;
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
    async fn read_function(f: String, client: &AsyncClient) -> Result<CargoToml, Box<dyn Error>> {
        let mut file = File::open(&f).await?;
        let metadata = file.metadata().await?;
        assert!(metadata.is_file(), "{} is not a file", f);
        let mut buffer = Vec::with_capacity(metadata.size() as usize);
        while file.read_buf(&mut buffer).await? > 0 {}
        let cargo_toml = CargoToml::from_str(&String::from_utf8(buffer)?)?;
        let krate = client.get_crate(&cargo_toml.package.name).await?;
        if krate
            .versions
            .into_iter()
            .any(|v| v.num == cargo_toml.package.version)
        {
            return Err(format!(
                "{} version {} is already on crates.io",
                &cargo_toml.package.name, &cargo_toml.package.version
            )
            .into());
        }
        Ok(cargo_toml)
    }

    let opt: Opt = Opt::from_args();
    let client = AsyncClient::new("cruiser-ci (buzzec@buzzec.net)", Duration::from_secs(1))?;

    let files = try_join_all(opt.file.into_iter().map(|f| read_function(f, &client))).await?;
    for cargo_toml in &files {
        for dependency in &cargo_toml.dependencies {
            for other in &files {
                if dependency.name == other.package.name
                    && dependency.version != other.package.version
                {
                    return Err(format!(
                        "Mismatched version of {} in {}",
                        dependency.name, cargo_toml.package.name
                    )
                    .into());
                }
            }
        }
    }

    Ok(())
}

pub struct CargoToml {
    pub package: CargoPackage,
    pub dependencies: Vec<CargoDependency>,
}
impl FromStr for CargoToml {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<Value>()?;
        let package = CargoPackage::try_from(
            value
                .get("package")
                .ok_or("cargo.toml does not have a package section")?,
        )?;
        let dependencies = value
            .get("dependencies")
            .ok_or("cargo.toml does not have a dependencies section")?
            .as_table()
            .ok_or("dependencies section is not a table")?
            .into_iter()
            .chain(
                value
                    .get("build-dependencies")
                    .map(|d| {
                        d.as_table()
                            .map(IntoIterator::into_iter)
                            .into_iter()
                            .flatten()
                    })
                    .into_iter()
                    .flatten(),
            )
            .map(|(name, value)| CargoDependency::new(name.clone(), value))
            .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
        Ok(Self {
            package,
            dependencies,
        })
    }
}

pub struct CargoPackage {
    pub name: String,
    pub version: String,
}
impl<'a> TryFrom<&'a Value> for CargoPackage {
    type Error = Box<dyn Error>;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value
                .get("name")
                .ok_or("cargo.toml does not have a name")?
                .as_str()
                .ok_or("name is not a string")?
                .to_string(),
            version: value
                .get("version")
                .ok_or("cargo.toml does not have a version")?
                .as_str()
                .ok_or("version is not a string")?
                .to_string(),
        })
    }
}

pub struct CargoDependency {
    pub name: String,
    pub version: String,
}
impl CargoDependency {
    pub fn new(name: String, value: &Value) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            name,
            version: if value.is_str() {
                value
            } else {
                value
                    .as_table()
                    .ok_or("dependency is not table or string")?
                    .get("version")
                    .ok_or("dependency does not have a version")?
            }
            .as_str()
            .ok_or("dependency version is not a string")?
            .to_string(),
        })
    }
}
