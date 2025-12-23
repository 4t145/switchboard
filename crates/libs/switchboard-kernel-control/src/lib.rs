pub mod kernel {
    tonic::include_proto!("switchboard.kernel");
}
pub use tonic_health;
mod type_convert;
pub use type_convert::TryFromProtoKernelStateError;

