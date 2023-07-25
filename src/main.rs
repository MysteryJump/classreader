use std::{path::Path, process::exit};

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
                    .filter(|e| e.path().extension().unwrap() == "class")
                    .map(|e| e.path().to_str().unwrap().to_owned())
                    .collect::<Vec<_>>()
            } else {
                vec![x]
            }
        })
        .collect::<Vec<_>>();

    for p in &paths {
        let class_file = std::fs::read(p).unwrap();
        let (_, c) = parse_class_file(&class_file).unwrap();
        extract_component(
            &c,
            &ExtractorContext {
                target_access_modifiers: AccessModifier::empty(),
            },
        );
        println!("Successfully parsed class file: {:?}", p);
    }
}
