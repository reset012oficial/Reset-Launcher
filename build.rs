#[cfg(windows)]
fn main() {
    use std::path::Path;

    // Set the subsystem to windows (GUI) to hide the console window
    // This only affects the final .exe, not cargo run
    let mut res = winresource::WindowsResource::new();

    // Set the icon if it exists (convert Path to string for set_icon)
    let icon_path = Path::new("assets/icon.ico");
    if icon_path.exists() {
        res.set_icon(icon_path.to_str().unwrap());
    }

    // Set the manifest for DPI awareness and Windows version
    res.set_manifest(
        r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <assemblyIdentity version="1.0.0.0" name="Reset Launcher" type="win32" />
  <description>Reset Launcher for Horizon (Minecraft Rust Edition)</description>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="asInvoker" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
  <application xmlns="urn:schemas-microsoft-com:asm.v3">
    <windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">PerMonitorV2,PerMonitor</dpiAwareness>
    </windowsSettings>
  </application>
</assembly>
"#,
    );

    res.compile().expect("Failed to compile Windows resource");
}

#[cfg(not(windows))]
fn main() {
    // Nothing to do on non-Windows platforms
}