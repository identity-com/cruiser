#![warn(unused_import_braces, unused_imports, clippy::pedantic)]

use ctrlc::set_handler;
use futures::executor::block_on;
use lazy_static::lazy_static;
use pbr::MultiBar;
use prettytable::{cell, row, Table};
use std::collections::HashMap;
use std::error::Error;
use std::io::stderr;
use std::process::{exit, Stdio};
use std::str::FromStr;
use std::time::Duration;
use structopt::StructOpt;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tokio::task::spawn_blocking;
use tokio::time::sleep;

#[derive(StructOpt)]
#[structopt(
    name = "check_all",
    about = "Runs clippy on all combinations of features"
)]
struct Opt {
    #[structopt(short, long)]
    verbose: bool,
    #[structopt(short, long)]
    package: Option<String>,
    #[structopt(short, long)]
    feature: Option<Vec<Feature>>,
}

#[derive(Clone)]
struct Feature {
    feature: String,
    dependants: Vec<String>,
}
impl FromStr for Feature {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(':');
        let feature = split.next().ok_or("No feature provided")?.to_string();
        let dependants = split
            .next()
            .map(|val| val.split(',').map(ToString::to_string).collect())
            .unwrap_or_default();
        if split.next().is_some() {
            Err("Too many colons".into())
        } else {
            Ok(Self {
                feature,
                dependants,
            })
        }
    }
}

lazy_static! {
    static ref CHILDREN: Mutex<HashMap<&'static str, Child>> = Mutex::new(HashMap::new());
}

#[allow(clippy::too_many_lines)]
#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    let features = opt.feature.clone().unwrap_or_default();
    let mut total_runs =
        2_usize.pow(u32::try_from(features.len()).expect("No way we can run that many features"));
    // let mut doc_pb = mb.create_bar(total_runs as u64);
    // doc_pb.format("[=>_]");
    // doc_pb.show_message = true;
    // doc_pb.message("`cargo doc`    ");

    set_handler(|| {
        for child in block_on(CHILDREN.lock()).values_mut() {
            block_on(child.kill()).expect("Could not kill child");
            println!("Exited child");
        }
        exit(1);
    })
    .expect("Could not set handler");

    for dependant in features.iter().flat_map(|feature| &feature.dependants) {
        assert!(
            features.iter().any(|feature| &feature.feature == dependant),
            "Unknown dependant: `{}`",
            dependant
        );
    }
    let feature_matrix: Vec<Vec<_>> = (0..total_runs)
        .filter_map(|val| {
            let list = features
                .iter()
                .enumerate()
                .filter_map(|(index, feature)| {
                    if val & (1 << index) > 0 {
                        Some(feature.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            for dependant in list.iter().flat_map(|feature| &feature.dependants) {
                if !list.iter().any(|feature| &feature.feature == dependant) {
                    return None;
                }
            }
            Some(list.into_iter().map(|feature| feature.feature).collect())
        })
        .collect();
    total_runs = feature_matrix.len();

    let mb = MultiBar::new();
    mb.println("Running checks: ");
    let mut clippy_pb = mb.create_bar(total_runs as u64);
    clippy_pb.format("[=>_]");
    clippy_pb.show_message = true;
    clippy_pb.message("`cargo clippy` ");
    let mb = spawn_blocking(move || mb.listen());

    let mut clippy_results = Vec::new();
    for features in feature_matrix {
        let mut command = Command::new("cargo");
        command
            .arg("clippy")
            .arg("--tests")
            .arg("--examples")
            .arg("--no-default-features");
        if opt.verbose {
            command.arg("--verbose");
        }
        if let Some(package) = &opt.package {
            command.arg("-p").arg(package);
        }
        for feature in &features {
            command.arg("--features").arg(feature);
        }
        command
            .arg("--")
            .arg("--deny=warnings")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = command.spawn().expect("Could not start command");
        let stdout = child.stdout.take().expect("Could not take stdout of child");
        let stderr = child.stderr.take().expect("Could not take stderr of child");

        assert!(
            CHILDREN.lock().await.insert("clippy", child).is_none(),
            "Duplicate `clippy` instance"
        );
        let exit_status = loop {
            let exit_status = CHILDREN.lock().await.get_mut("clippy").unwrap().try_wait();
            if let Some(val) = exit_status.map_or_else(|err| Some(Err(err)), |val| val.map(Ok)) {
                break val;
            }
            sleep(Duration::from_millis(500)).await;
        };
        CHILDREN
            .lock()
            .await
            .remove("clippy")
            .expect("Could not find clippy instance");
        clippy_results.push(match exit_status {
            Err(e) => Err((features, Err(e))),
            Ok(status) if status.success() => Ok(status),
            Ok(status) => Err((features, Ok((status, stdout, stderr)))),
        });

        clippy_pb.inc();
    }

    clippy_pb.finish_print("`cargo clippy` complete!");
    mb.await.expect("Could not join");

    let mut successes = Vec::new();
    let mut clippy_errors = Vec::new();
    let mut other_errors = Vec::new();
    for result in clippy_results {
        match result {
            Ok(status) => successes.push(status),
            Err((features, Ok((status, stdout, stderr)))) => {
                println!("Features: {:?}, status: {}", features, status);
                println!("stdout:");
                let mut reader = BufReader::new(stdout).lines();
                while let Some(line) = reader.next_line().await.expect("Could not read line") {
                    println!("{}", line);
                }
                println!("stderr:");
                let mut reader = BufReader::new(stderr).lines();
                while let Some(line) = reader.next_line().await.expect("Could not read line") {
                    println!("{}", line);
                }

                clippy_errors.push(features);
            }
            Err((features, Err(error))) => {
                println!("Features: {:?}", features);
                println!("    error: {}", error);
                other_errors.push(features);
            }
        }
    }

    println!();
    println!("Summary:");
    let mut table = Table::new();
    table.add_row(row!["Successful Runs", successes.len()]);
    table.add_row(if clippy_errors.is_empty() {
        row![Fg => "Clippy Errors", clippy_errors.len()]
    } else {
        row![Fr => "Clippy Errors", clippy_errors.len()]
    });
    table.add_row(if other_errors.is_empty() {
        row![Fg => "Other Errors", other_errors.len()]
    } else {
        row![Fr => "Other Errors", other_errors.len()]
    });
    table.printstd();

    let exit_code = if clippy_errors.is_empty() && other_errors.is_empty() {
        0
    } else {
        1
    };

    if !clippy_errors.is_empty() {
        println!();
        let mut table = Table::new();
        table.add_row(row!["Clippy Errors"]);
        for features in clippy_errors {
            table.add_row(row![Fr => format!("{:?}", features)]);
        }
        table.print(&mut stderr()).expect("Could not print");
    }

    if !other_errors.is_empty() {
        println!();
        let mut table = Table::new();
        table.add_row(row!["Other Errors"]);
        for features in other_errors {
            table.add_row(row![Fr => format!("{:?}", features)]);
        }
        table.print(&mut stderr()).expect("Could not print");
    }

    exit(exit_code);
}
