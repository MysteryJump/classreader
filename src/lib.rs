pub mod class_file;
pub mod component;
pub mod descriptor;
pub mod extractor;
pub mod proto;
pub mod signature;

pub use prost::bytes;

use prost::{bytes::Buf, Message};
use robusta_jni::bridge;

#[bridge]
pub mod jni {
    use prost::Message;
    use robusta_jni::convert::Signature;
    use std::error::Error;

    use crate::extractor;

    #[derive(Signature)]
    #[package(com.github.nreopigs.classreader)]
    struct ClassReader;

    impl ClassReader {
        #[allow(deprecated)]
        #[allow(clippy::needless_borrow)]
        pub extern "jni" fn extractFromJarPath(jar_path: String) -> Vec<i8> {
            let components = match extractor::extract_members_from_jar(jar_path) {
                Ok(c) => c,
                Err(e) => {
                    println!("Error: {}", e.description());
                    return vec![];
                }
            };

            let component: crate::proto::component::ComponentList = (&components).into();
            let mut encoded_buf = Vec::new();
            component.encode(&mut encoded_buf).unwrap();

            encoded_buf.iter().map(|x| *x as i8).collect()
        }

        #[allow(deprecated)]
        #[allow(clippy::needless_borrow)]
        pub extern "jni" fn test(a: String) -> String {
            a
        }
    }
}

pub fn decode_component_list<B: Buf>(buf: B) -> Result<proto::component::ComponentList, String> {
    proto::component::ComponentList::decode(buf).map_err(|e| e.to_string())
}
