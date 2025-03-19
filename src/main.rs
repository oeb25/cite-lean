use std::collections::HashMap;

use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser as _;
use color_eyre::eyre::eyre;
use tracing::{debug, error, info, info_span};
use tracing_subscriber::prelude::*;

type Result<T, E = color_eyre::Report> = std::result::Result<T, E>;

#[derive(Debug, clap::Parser)]
struct Cli {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    DownloadData {
        #[clap(long, short)]
        doc_url: String,
    },
    CiteLean {
        #[clap(long, default_value_t = false)]
        write: bool,
        root: Utf8PathBuf,
    },
}

#[derive(Debug, serde::Deserialize)]
struct Data {
    // modules: HashMap<String, serde_json::Value>,
    declarations: HashMap<String, Declaration>,
    // #[serde(flatten)]
    // rest: HashMap<String, serde_json::Value>,
}

#[derive(Debug, bincode::Decode, bincode::Encode)]
struct FastData {
    declarations: HashMap<String, String>,
}

#[derive(Debug, serde::Deserialize)]
struct Declaration {
    #[serde(rename = "docLink")]
    doc_link: String,
    // kind: String,
    // #[serde(flatten)]
    // rest: serde_json::Value,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    tracing_subscriber::Registry::default()
        .with(tracing_error::ErrorLayer::default())
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .without_time(),
        )
        .with(tracing_subscriber::filter::FilterFn::new(|m| {
            !m.target().contains("hyper")
        }))
        .init();

    let cli = Cli::parse();

    match &cli.cmd {
        Command::DownloadData { doc_url } => {
            let data = reqwest::blocking::get(format!(
                "{}/declarations/declaration-data.bmp",
                doc_url.trim_end_matches('/')
            ))?
            .bytes()?;
            let data: Data = serde_json::from_slice(&data)?;
            let fast_data = FastData {
                declarations: data
                    .declarations
                    .into_iter()
                    .map(|(k, v)| (k, v.doc_link))
                    .collect(),
            };
            bincode::encode_into_std_write(
                fast_data,
                &mut std::fs::File::create(".cite-lean.bin")?,
                bincode::config::standard(),
            )?;
        }
        Command::CiteLean { write, root } => {
            let data: FastData = bincode::decode_from_std_read(
                &mut std::fs::File::open(".cite-lean.bin")?,
                bincode::config::standard(),
            )?;

            let root = root.canonicalize_utf8()?;
            for f in walkdir::WalkDir::new(&root) {
                let f = f?;
                // skip anything but .tex
                if f.path().extension().map_or(true, |ext| ext != "tex") {
                    continue;
                }
                let file = Utf8Path::from_path(f.path()).unwrap();
                let p = file.strip_prefix(&root)?;
                let span = info_span!("file", %p);
                let _e = span.enter();
                debug!("processing...");
                let src = std::fs::read_to_string(f.path())?;

                let mut output = String::new();

                for (line_idx, line) in src.lines().enumerate() {
                    let needle = "cite-lean(";
                    if let Some(start) = line.find(needle) {
                        let end = line[start + needle.len()..]
                            .find(')')
                            .ok_or_else(|| eyre!("expected closing parenthesis"))?;
                        let key = &line[start + needle.len()..start + needle.len() + end];
                        let Some(doc_link) = data.declarations.get(key) else {
                            let link =
                                format!("{file}:{}:{}", line_idx + 1, start + needle.len() + 1);
                            error!(%key, "missing declaration {}", link);
                            let pre = line[0..start].find(|c: char| !c.is_whitespace());
                            output.push_str(&line[0..pre.unwrap_or(0)]);
                            output.push_str("\\leanMissing");
                            output.push_str(" % ");
                            output.push_str(&line[start..]);
                            output.push('\n');
                            continue;
                        };
                        info!(%key, %doc_link);

                        let (url, tag) =
                            doc_link.split_once('#').unwrap_or((doc_link.as_str(), ""));
                        let url = format!(
                            "{}\\#{}",
                            url.trim_start_matches(|c: char| "./".contains(c)),
                            urlencoding::encode(tag).replace('%', "\\%"),
                        );
                        let pre = line[0..start].find(|c: char| !c.is_whitespace());
                        output.push_str(&line[0..pre.unwrap_or(0)]);
                        output.push_str(&format!("\\lean{{{url}}}"));
                        output.push_str(" % ");
                        output.push_str(&line[start..]);
                        output.push('\n');
                    } else {
                        output.push_str(line);
                        output.push('\n');
                    }
                }

                if output == src {
                    debug!("no changes");
                    continue;
                }
                if *write {
                    std::fs::write(file, output)?;
                    info!(%file, "wrote");
                } else {
                    println!("{}", output);
                }
            }
        }
    }

    Ok(())
}
