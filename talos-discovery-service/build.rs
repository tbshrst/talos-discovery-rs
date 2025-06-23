fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(
            &[
                "proto/v1alpha1/client/affiliate.proto",
                "proto/v1alpha1/server/cluster.proto",
            ],
            &["proto/v1alpha1/client", "proto/v1alpha1/server", "proto"],
        )?;
    Ok(())
}
