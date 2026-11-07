//! Embeds the application icon and version details into the Windows
//! executable, so Explorer and the taskbar show the Chronote icon.

fn main() {
    println!("cargo:rerun-if-changed=assets/icon/chronote.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource
            .set_icon("assets/icon/chronote.ico")
            .set("ProductName", "Chronote")
            .set("FileDescription", "Chronote: study timelines with notes");
        resource
            .compile()
            .expect("failed to embed the Windows icon resource");
    }
}
