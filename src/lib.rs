pub mod class_file;
pub mod component;
pub mod descriptor;
pub mod proto;
pub mod signature;
use robusta_jni::bridge;

#[bridge]
pub mod jni {

    use robusta_jni::convert::Signature;
    use std::error::Error;

    #[derive(Signature)]
    #[package(com.github.nreopigs.classreader)]
    struct ClassReader;

    impl ClassReader {
        #[allow(deprecated)]
        #[allow(clippy::needless_borrow)]
        pub extern "jni" fn extractFromJarPath(jar_path: String) -> Vec<i8> {
            jar_path.as_bytes().iter().map(|&x| x as i8).collect()
        }

        #[allow(deprecated)]
        #[allow(clippy::needless_borrow)]
        pub extern "jni" fn test(a: String) -> String {
            a
        }
    }
}
