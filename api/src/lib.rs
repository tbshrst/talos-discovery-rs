pub mod discovery {
    tonic::include_proto!("sidero.discovery.server");
}

pub use discovery::*;
pub use prost;
pub use tonic;
