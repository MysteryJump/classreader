use class_file::parse_class_file;
use component::extract_component;

use crate::component::{AccessModifier, ExtractorContext};

mod class_file;
mod component;
mod descriptor;
mod signature;

fn main() {
    // let class_file_path = "../fernflower/compiled/java.base/classes/java/util/ArrayList.class";
    // let class_file_path = "Main.class";
    // let class_file_path = "tmp/spring-core-6.0.11/org/springframework/core/AttributeAccessor.class";
    let class_file_path = "../../playground/sdl-jmods/java.base/classes/java/util/HashMap.class";
    let class_file = std::fs::read(class_file_path).unwrap();

    let (_, class_file) = parse_class_file(&class_file).unwrap();
    // class_file.methods.iter().for_each(|method| {
    //     println!("{}", method.get_name(&class_file.constant_pool));
    // });

    let extractor_ctxt = ExtractorContext {
        target_access_modifiers: AccessModifier::empty(),
    };

    let component = extract_component(&class_file, &extractor_ctxt);
    println!("{:#?}", component);
}
