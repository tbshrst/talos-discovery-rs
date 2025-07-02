fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(
            &["v1alpha1/client/affiliate.proto", "v1alpha1/server/cluster.proto"],
            &["discovery-api/api"],
        )?;
    Ok(())
}
