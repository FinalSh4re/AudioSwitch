fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_toolkit_path(r#"C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64"#);
    res.set_manifest(
        r#"
    <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
    <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        <security>
            <requestedPrivileges>
                <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
            </requestedPrivileges>
        </security>
    </trustInfo>
    </assembly>
    "#,
    );
    res.set_icon("icon.ico");
    res.compile()
        .expect("Failed to compile binary with manifest.");
}
