pub mod class_file;
pub mod component;
pub mod descriptor;
pub mod extractor;
pub mod proto;
pub mod signature;
use robusta_jni::bridge;

#[bridge]
pub mod jni {

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
                Err(_) => {
                    return vec![];
                }
            };

            components.iter().map(|x| *x as i8).collect()
        }

        #[allow(deprecated)]
        #[allow(clippy::needless_borrow)]
        pub extern "jni" fn test(a: String) -> String {
            a
        }
    }
}