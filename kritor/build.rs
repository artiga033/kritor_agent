use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut protos = vec![];
    let sub_folders = fs::read_dir("kritor/protos")?;
    for f in sub_folders.filter(|x| x.as_ref().is_ok_and(|x| x.path().is_dir())) {
        let f = f?;
        let path = f.path();
        for f in fs::read_dir(path)? {
            let f = f?;
            let path = f.path();
            if let Some(ext) = path.extension() {
                if ext == "proto" {
                    protos.push(path);
                }
            }
        }
    }
    let descriptor_file = PathBuf::from(env::var("OUT_DIR").unwrap()).join("descriptor_set.bin");
    let mut builder = tonic_build::configure();
    builder = builder.file_descriptor_set_path(descriptor_file);
    builder =
        builder.generate_default_stubs(env::var("CARGO_FEATURE_GENERATE_DEFAULT_STUBS").is_ok());
    #[cfg(feature = "server")]
    {
        builder = builder.build_server(true);
    }
    #[cfg(feature = "client")]
    {
        builder = builder.build_client(true);
    }
    builder.compile(protos.as_slice(), &["kritor/protos"])?;
    Ok(())
}
