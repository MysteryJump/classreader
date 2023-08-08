use std::{error::Error, io::Read, path::Path};

use crate::{
    class_file::parse_class_file,
    component::{extract_component, AccessModifier, Component, ExtractorContext},
};

pub fn extract_members_from_jar<P: AsRef<Path>>(path: P) -> Result<Vec<Component>, Box<dyn Error>> {
    let path = path.as_ref();

    if let Some(ext) = path.extension() {
        if ext != "jar" && ext != "jmod" {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path must be a JAR file",
            )));
        }
    } else {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Path must be a JAR file",
        )));
    }

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            return Err(Box::new(e));
        }
    };

    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(e) => {
            return Err(Box::new(e));
        }
    };

    let mut components = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        if !file.name().ends_with(".class") {
            continue;
        }

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

        components.push(comp);
    }

    Ok(components)
}
