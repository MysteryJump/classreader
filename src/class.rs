use crate::class_file::{AttributeInfo, ClassFile, ConstantPoolInfo};

struct Class {
    info: ClassInfo,
}

struct ClassInfo {
    minor_version: u16,
    major_version: u16,
}

struct ClassBuilder {
    class_file: ClassFile,
}

impl ClassBuilder {
    pub fn new(class_file: ClassFile) -> Self {
        Self { class_file }
    }

    pub fn build(&self) -> Class {
        let info = ClassInfo {
            minor_version: self.class_file.minor_version,
            major_version: self.class_file.major_version,
        };
        Class { info }
    }

    pub fn read_attribute(&self, attribute_info: AttributeInfo) {
        let attribute_name = self
            .class_file
            .constant_pool
            .get(attribute_info.attribute_name_index as usize)
            .unwrap();
        let ConstantPoolInfo::Utf8 { utf8_str: attribute_name, length: _, bytes: _ } = attribute_name else {
            panic!("Expected Utf8 constant pool info");
        };

        match attribute_name.as_str() {
            "SourceFile" => {
                let source_file_attribute = self
                    .class_file
                    .constant_pool
                    .get(attribute_info.attribute_name_index as usize)
                    .unwrap();
                let ConstantPoolInfo::Utf8 { utf8_str: source_file_attribute, length: _, bytes: _ } = source_file_attribute else {
                    panic!("Expected Utf8 constant pool info");
                };
            }
            _ => todo!(),
        }

        todo!()
    }
}
