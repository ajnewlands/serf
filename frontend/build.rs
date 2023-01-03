fn main() {
    if cfg!(target_os = "windows") {
        println!("cargo:rerun-if-changed=../icons/serf.ico");
        let mut res = winres::WindowsResource::new();
        res.set_icon("../icons/serf.ico");
        res.compile().unwrap();
    }
}
