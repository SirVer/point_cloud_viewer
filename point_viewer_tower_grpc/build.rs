extern crate tower_grpc_build;

fn main() {
    tower_grpc_build::Config::new()
        .enable_server(true)
        .enable_client(true)
        .build(&["../point_viewer_grpc_proto_rust/src/proto.proto"], &[".."])
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}
