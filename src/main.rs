use std::{
    io::Read,
    path::{Path, PathBuf},
    process::exit,
};

use class_file::parse_class_file;
use component::extract_component;

use crate::component::{AccessModifier, ExtractorContext};

mod class_file;
mod component;
mod descriptor;
mod signature;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        eprintln!("Usage: {} <class file path>", args[0]);
        exit(1);
    }

    let paths = args
        .into_iter()
        .skip(1)
        .flat_map(|x| {
            let p = Path::new(&x);
            if p.is_dir() {
                walkdir::WalkDir::new(p)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_file())
                    .filter(|e| {
                        let ext = e.path().extension().unwrap();
                        ext == "class" || ext == "jar"
                    })
                    .map(|e| e.path().to_path_buf())
                    .collect::<Vec<_>>()
            } else {
                vec![x.into()]
            }
        })
        .collect::<Vec<_>>();

    for p in &paths {
        if p.ends_with(".jar") {
            let file = match std::fs::File::open(p) {
                Ok(f) => f,
                Err(e) => {
                    println!("Error opening JAR file: {:?}", e);
                    continue;
                }
            };

            let mut archive = match zip::ZipArchive::new(file) {
                Ok(a) => a,
                Err(e) => {
                    println!("Error opening JAR file: {:?}", e);
                    continue;
                }
            };

            for i in 0..archive.len() {
                let mut file = archive.by_index(i).unwrap();
                let mut buf = Vec::new();
                file.read_to_end(&mut buf).unwrap();

                let component = parse_class_file(&buf).map(|(_, c)| {
                    extract_component(
                        &c,
                        &ExtractorContext {
                            target_access_modifiers: AccessModifier::empty(),
                        },
                    )
                });

                let comp = if let Err(e) = component {
                    println!("Error parsing class file: {:?}", e);
                    continue;
                } else {
                    component.unwrap()
                };
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

            println!("{comp:#?}");

            println!("Successfully parsed class file: {:?}", p);
        }
    }
}
