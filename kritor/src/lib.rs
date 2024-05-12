pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("descriptor_set");

pub mod auth {
    tonic::include_proto!("kritor.authentication");
}
pub mod common {
    tonic::include_proto!("kritor.common");
}
pub mod core {
    tonic::include_proto!("kritor.core");
}
pub mod developer {
    tonic::include_proto!("kritor.developer");
}
pub mod event {
    tonic::include_proto!("kritor.event");
}
pub mod file {
    tonic::include_proto!("kritor.file");
}
pub mod friend {
    tonic::include_proto!("kritor.friend");
}
pub mod group {
    tonic::include_proto!("kritor.group");
}
pub mod guild {
    tonic::include_proto!("kritor.guild");
}
pub mod message {
    tonic::include_proto!("kritor.message");
}
pub mod process {
    tonic::include_proto!("kritor.process");
}
pub mod reverse {
    tonic::include_proto!("kritor.reverse");
}
pub mod web {
    tonic::include_proto!("kritor.web");
}
