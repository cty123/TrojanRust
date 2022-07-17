pub mod grpc_stream;

pub mod grpc_transport {
    tonic::include_proto!("trojan_rust.transport.grpc");
}