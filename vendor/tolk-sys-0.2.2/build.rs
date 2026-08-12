use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let target = env::var("TARGET").unwrap();
    let libs = if target.contains("64") {
        format!("{}/vendor/tolk/libs/x64", manifest)
    } else {
        format!("{}/vendor/tolk/libs/x86", manifest)
    };
    let mut target: PathBuf = out_dir.into();
    target.pop();
    target.pop();
    target.pop();
    let target = target.into_os_string().into_string().unwrap();
    for entry in fs::read_dir(libs)? {
        let path = entry.unwrap().path();
        let file_name = path
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .unwrap();
        let out = format!("{}/{}", &target, &file_name);
        fs::copy(&path, out)?;
        let out = format!("{}/examples/{}", &target, &file_name);
        fs::copy(&path, out)?;
    }
    let root = "vendor/tolk";
    cc::Build::new()
        .cpp(true)
        .define("_EXPORTING", "")
        .define("UNICODE", "")
        .file(format!("{}/src/Tolk.cpp", root))
        .file(format!("{}/src/ScreenReaderDriverJAWS.cpp", root))
        .file(format!("{}/src/ScreenReaderDriverNVDA.cpp", root))
        .file(format!("{}/src/ScreenReaderDriverSA.cpp", root))
        .file(format!("{}/src/ScreenReaderDriverSNova.cpp", root))
        .file(format!("{}/src/ScreenReaderDriverWE.cpp", root))
        .file(format!("{}/src/ScreenReaderDriverZT.cpp", root))
        .file(format!("{}/src/ScreenReaderDriverSAPI.cpp", root))
        .file(format!("{}/src/fsapi.c", root))
        .file(format!("{}/src/wineyes.c", root))
        .file(format!("{}/src/zt.c", root))
        .compile("tolk");
    println!("cargo:rustc-flags=-l User32 -l Ole32 -l OleAut32");
    Ok(())
}
