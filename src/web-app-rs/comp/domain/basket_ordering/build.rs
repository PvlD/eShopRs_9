fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "ssr")]
    unsafe {
        std::env::set_var("PROTOC", "C:/soft/protoc-win64/bin/protoc.exe")
    };

    #[cfg(feature = "ssr")]
    tonic_prost_build::compile_protos("./api/protos/basket.proto")?;
    Ok(())
}
