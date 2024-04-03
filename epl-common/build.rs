use std::io::Result;
fn main() -> Result<()> {
    prost_build::compile_protos(&["src/protobufs/raw/PreloadedUserSettings.proto", "src/protobufs/raw/FrecencyUserSettings.proto"], &["src/protobufs/raw/"])?;
    Ok(())
}