use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["src/noise/proto/handshake_payload.proto"], &["src/noise"])?;
    Ok(())
}
