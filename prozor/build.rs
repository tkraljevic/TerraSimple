#![forbid(unsafe_code)]

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=../resursi/ikone/izdanje/TerraSimple.ico");
    #[cfg(windows)]
    {
        winresource::WindowsResource::new()
            .set_icon("../resursi/ikone/izdanje/TerraSimple.ico")
            .set("ProductName", "TerraSimple")
            .set("FileDescription", "TerraSimple")
            .set("OriginalFilename", "TerraSimple.exe")
            .compile()?;
    }
    Ok(())
}
