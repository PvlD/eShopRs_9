fn main() {
    #[cfg(feature = "ssr")]
    {
        std::env::set_var("PROTOC", "C:/soft/protoc-win64/bin/protoc.exe");
        tonic_prost_build::compile_protos("proto/basket.proto")
            .expect("compiling basket.proto");
    }
}
