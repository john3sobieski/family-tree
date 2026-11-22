// Embeds the icon and the application manifest into famtree.exe (Windows only).

const MANIFEST: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <compatibility xmlns="urn:schemas-microsoft-com:compatibility.v1">
    <application>
      <!-- Windows 10 and Windows 11 share this id. -->
      <supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>
    </application>
  </compatibility>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2,PerMonitor</dpiAwareness>
      <longPathAware xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">true</longPathAware>
    </windowsSettings>
  </application>
</assembly>
"#;

fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    res.set("ProductName", "FamTree");
    res.set("FileDescription", "FamTree");
    if let Err(e) = res.compile() {
        println!("cargo:warning=icon not embedded: {e}");
    }

    // The MSVC linker merges this with rustc's own default manifest (asInvoker).
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        let out = std::env::var("OUT_DIR").expect("OUT_DIR is set by cargo");
        let path = std::path::Path::new(&out).join("famtree.manifest");
        match std::fs::write(&path, MANIFEST) {
            Ok(()) => {
                println!("cargo:rustc-link-arg-bins=/MANIFEST:EMBED");
                println!("cargo:rustc-link-arg-bins=/MANIFESTINPUT:{}", path.display());
            }
            Err(e) => println!("cargo:warning=manifest not embedded: {e}"),
        }
    }
}
