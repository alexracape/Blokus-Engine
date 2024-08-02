use std::io;

fn main() -> io::Result<()> {
    // tonic_build::compile_protos("../proto/model.proto")?;
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(&["model.proto"], &["../proto/"])
    // Ok(())
}
