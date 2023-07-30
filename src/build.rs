fn main() -> std::io::Result<()> {
    prost_build::compile_protos(&["src/protos/component.proto"], &["src/protos"])?;
    Ok(())
}
