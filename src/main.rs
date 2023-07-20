use class_file::parse_class_file;

mod class;
mod class_file;

fn main() {
    let class_file_path = "java.base/classes/java/util/ArrayList.class";
    let class_file = std::fs::read(class_file_path).unwrap();

    let class_file = parse_class_file(&class_file).unwrap();
}
