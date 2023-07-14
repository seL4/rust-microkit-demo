use std::process::Command;
use std::env;
fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    
    //let out_dir = env::var("OUT_DIR").unwrap();
    //let out_path = PathBuf::from(&out_dir).join("uartps.a");
    //fs::write(&out_path, &priv_key_pem).unwrap();
    Command::new("make")
        .args(["clean"])
        .output()
        .expect("failed to execute process");
    Command::new("make")
        .args(["libuartps.a"])
        .output()
        .expect("failed to execute process");
    Command::new("cp")
        .args(["libuartps.a", &(out_dir + "/.") ])
        .output()
        .expect("failed to execute process");
    Command::new("cp")
        .args(["libuartps.a", "/work/build/target/release/deps/." ])
        .output()
        .expect("failed to execute process");

    //println!("cargo:rustc-link-lib=libdummy.a");
}