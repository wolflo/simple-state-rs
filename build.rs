use serde::Deserialize;
use std::{env, fs, path::Path};

pub const SIMPLESTATE_JSON_PATH: &'static str = "./out/SimpleState.sol/SimpleState.json";

#[derive(Deserialize)]
pub struct ContractJson {
    pub abi: serde_json::Value,
    pub bin: String,
}

fn main() {
    let out_dir_base = env::current_dir().unwrap();
    let out_dir = Path::new(&out_dir_base).join("build");
    if !out_dir.exists() {
        fs::create_dir(&out_dir).expect("Can't create out_dir.");
    }

    let raw = fs::read_to_string(&SIMPLESTATE_JSON_PATH).unwrap();
    let json: ContractJson = serde_json::from_str(&raw).unwrap();

    let bytecode = json.bin.trim_start_matches("0x");
    fs::write(&out_dir.clone().join("SimpleState.bin.json"), &bytecode).unwrap();
    fs::write(
        &out_dir.clone().join("SimpleState.abi.json"),
        &json.abi.to_string(),
    )
    .unwrap();

    println!(
        "cargo:rustc-env=SOLC_BUILD_DIR={}",
        out_dir.into_os_string().into_string().unwrap()
    );

    // Tell Cargo to rerun build script anytime forge build outputs change
    println!("cargo:rerun-if-changed=out/");
}
