use std::{
    error::Error,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
    process::exit,
};

use class_file::parse_class_file;
use component::extract_component;
use extractor::extract_members_from_jar;

use clap::{Args, Parser};
use prost::Message;
use rayon::prelude::*;

use crate::component::{AccessModifier, ExtractorContext};

mod class_file;
mod component;
mod descriptor;
mod extractor;
mod proto;
mod signature;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    output_kind: OutputKind,

    /// The target class files or JAR files to parse
    #[arg(required = true)]
    input_paths: Vec<String>,

    /// The output path to write the parsed class files to (default: current directory).
    /// It must be a directory path.
    #[arg(short, long)]
    output_path: Option<String>,

    /// Whether to print the time taken to parse all files (default: false)
    #[arg(short, long, default_value_t = false)]
    time: bool,

    /// Whether to parse files in parallel (default: true)
    #[arg(short, long, default_value_t = true)]
    parallel: bool,
}

#[derive(Args, Debug)]
#[group(multiple = false)]
struct OutputKind {
    /// Whether to output the parsed class files as JSON (default: false, conflicts with proto)
    #[arg(short, long, default_value_t = false)]
    json: bool,

    /// Whether to output the parsed class files as Protocol Buffers (default: true, conflicts with json)
    #[arg(short, long, default_value_t = true)]
    proto: bool,
}

#[derive(Debug, Clone, Copy)]
enum OKind {
    Json,
    Proto,
}

fn main() {
    let args = Cli::parse();
    let output_kind = if args.output_kind.json {
        OKind::Json
    } else {
        OKind::Proto
    };

    let start_time = if args.time {
        Some(std::time::Instant::now())
    } else {
        None
    };

    let paths = args
        .input_paths
        .into_iter()
        .flat_map(|x| {
            let p = Path::new(&x);
            if p.is_dir() {
                walkdir::WalkDir::new(p)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_file())
                    .filter(|e| {
                        let ext = e.path().extension().unwrap();
                        ext == "class" || ext == "jar" || ext == "jmod"
                    })
                    .map(|e| e.path().to_path_buf())
                    .collect::<Vec<_>>()
            } else {
                vec![x.into()]
            }
        })
        .collect::<Vec<_>>();

    let output_dir = args
        .output_path
        .as_ref()
        .map(Path::new)
        .unwrap_or_else(|| Path::new("."));

    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir).unwrap();
    }

    if output_dir.is_file() {
        println!("Error: Output path must be a directory");
        exit(1);
    }

    if args.parallel {
        paths
            .par_iter()
            .for_each(|p| match extract_from_path(p, output_dir, output_kind) {
                Ok(_) => {}
                Err(err) => {
                    println!("Error: {}", err);
                }
            });
    } else {
        for p in &paths {
            match extract_from_path(p, output_dir, output_kind) {
                Ok(_) => {}
                Err(err) => {
                    println!("Error: {}", err);
                    continue;
                }
            }
        }
    }

    if let Some(start_time) = start_time {
        let elapsed = start_time.elapsed();
        println!(
            "Parsed {} files in {}.{:03}s",
            paths.len(),
            elapsed.as_secs(),
            elapsed.subsec_millis()
        );
    }
}

fn extract_from_path(
    p: &PathBuf,
    output_dir: &Path,
    output_kind: OKind,
) -> Result<(), Box<dyn Error>> {
    let output_ext = match output_kind {
        OKind::Json => "json",
        OKind::Proto => "pb",
    };

    let ext = p.extension().unwrap_or_default();
    let file_name = p.file_name().unwrap().to_str().unwrap();
    let output_path = output_dir.join(format!("{file_name}.{output_ext}",));

    if ext == "jar" || ext == "jmod" {
        let components = match extract_members_from_jar(p) {
            Ok(c) => c,
            Err(err) => {
                return Err(err);
            }
        };

        let mut writer = File::create(output_path).unwrap();

        match output_kind {
            OKind::Json => {
                serde_json::to_writer(writer, &components).unwrap();
            }
            OKind::Proto => {
                let component: crate::proto::component::ComponentList = (&components).into();

                let mut encoded_buf = Vec::new();
                component.encode(&mut encoded_buf).unwrap();
                writer.write_all(&encoded_buf).unwrap();
            }
        }
    } else {
        let class_file = std::fs::read(p).unwrap();
        let (_, c) = parse_class_file(&class_file).unwrap();

        let comp = extract_component(
            &c,
            &ExtractorContext {
                target_access_modifiers: AccessModifier::empty(),
            },
        );

        let mut writer = File::create(output_path).unwrap();

        match output_kind {
            OKind::Json => {
                serde_json::to_writer(writer, &comp).unwrap();
            }
            OKind::Proto => {
                let component: crate::proto::component::Component = (&comp).into();

                let mut encoded_buf = Vec::new();
                component.encode(&mut encoded_buf).unwrap();
                writer.write_all(&encoded_buf).unwrap();
            }
        }
    }

    Ok(())
}
