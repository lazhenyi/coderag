use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

fn run_npm(args: &[&str], cwd: &str) {
    let mut cmd = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.args(["/C", "npm"]);
        c
    } else {
        Command::new("npm")
    };

    let status = cmd
        .args(args)
        .current_dir(cwd)
        .status()
        .expect("failed to run npm");

    if !status.success() {
        panic!("npm command failed: {:?}", args);
    }
}

fn find_all_files(path: PathBuf) -> Vec<PathBuf> {
    let mut files = vec![];
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        } else if path.is_dir() {
            files.extend(find_all_files(path));
        }
    }
    files
}

fn is_compressed_asset(path: &str) -> bool {
    path.ends_with(".png")
        || path.ends_with(".jpg")
        || path.ends_with(".jpeg")
        || path.ends_with(".gif")
        || path.ends_with(".webp")
        || path.ends_with(".avif")
        || path.ends_with(".ico")
        || path.ends_with(".woff")
        || path.ends_with(".woff2")
        || path.ends_with(".ttf")
        || path.ends_with(".otf")
        || path.ends_with(".eot")
        || path.ends_with(".zip")
        || path.ends_with(".gz")
        || path.ends_with(".br")
        || path.ends_with(".xz")
        || path.ends_with(".wasm")
}

const MIN_COMPRESS_SIZE: usize = 256;

fn main() {
    println!("cargo:rerun-if-changed=../../web/package.json");
    println!("cargo:rerun-if-changed=../../web/vite.config.js");
    println!("cargo:rerun-if-changed=../../web/src/");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let project_root = manifest_dir.parent().unwrap().parent().unwrap();
    let web_dir = project_root.join("web");

    println!("cargo:warning=Building frontend...");
    run_npm(&["run", "build"], web_dir.to_str().unwrap());

    let dist = web_dir.join("dist");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let blob_dir = out_dir.join("dist_blobs");
    fs::create_dir_all(&blob_dir).unwrap();

    let mut strs = vec![];
    for file in find_all_files(dist.clone()) {
        let key = file
            .strip_prefix(&dist)
            .unwrap()
            .components()
            .collect::<PathBuf>()
            .to_string_lossy()
            .replace('\\', "/");
        let safe_name = key.replace('/', "_").replace('\\', "_");
        let blob_path = blob_dir.join(&safe_name);
        let content = fs::read(&file).unwrap();
        fs::write(&blob_path, &content).unwrap();

        let hash = md5::compute(&content);
        let etag_literal = format!("\"{:x}\"", hash);
        let key_literal = format!("\"{}\"", key.replace('"', "\\\""));

        let (gz_literal, br_literal) =
            if !is_compressed_asset(&key) && content.len() >= MIN_COMPRESS_SIZE {
                let gz_path = blob_dir.join(format!("{}.gz", safe_name));
                let br_path = blob_dir.join(format!("{}.br", safe_name));

                let gz_data = {
                    let mut encoder =
                        flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
                    let _ = encoder.write_all(&content);
                    encoder.finish().unwrap()
                };
                let br_data = {
                    let mut encoder = brotli::CompressorWriter::new(Vec::new(), 4096, 11, 22);
                    let _ = encoder.write_all(&content);
                    encoder.flush().unwrap();
                    encoder.into_inner()
                };

                if gz_data.len() < content.len() {
                    fs::write(&gz_path, &gz_data).unwrap();
                }
                if br_data.len() < content.len() {
                    fs::write(&br_path, &br_data).unwrap();
                }

                let gz = if gz_data.len() < content.len() {
                    format!(
                        "Some((include_bytes!(\"dist_blobs/{}.gz\").to_vec(), \"gzip\"))",
                        safe_name
                    )
                } else {
                    "None".to_string()
                };
                let br = if br_data.len() < content.len() {
                    format!(
                        "Some((include_bytes!(\"dist_blobs/{}.br\").to_vec(), \"br\"))",
                        safe_name
                    )
                } else {
                    "None".to_string()
                };
                (gz, br)
            } else {
                ("None".to_string(), "None".to_string())
            };

        strs.push(format!(
            r#"    (
        {},
        include_bytes!("dist_blobs/{}"),
        {},
        {},
        {},
    ),"#,
            key_literal,
            safe_name,
            etag_literal,
            gz_literal,
            br_literal
        ));
    }

    let out_file = out_dir.join("frontend.rs");
    let generated = format!(
        "lazy_static::lazy_static! {{\n    pub static ref FRONTEND: Vec<(\n        &'static str,\n        &'static [u8],\n        &'static str,\n        Option<(Vec<u8>, &'static str)>,\n        Option<(Vec<u8>, &'static str)>,\n    )> = vec![\n{}\n    ];\n}}\n",
        strs.join("\n")
    );
    fs::write(&out_file, generated).unwrap();
    println!("cargo:include={}", out_file.display());
}
