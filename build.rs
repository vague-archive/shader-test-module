use std::path::Path;

fn main() {
    println!("Performing FFI codegen...");
    let current_dir_os_string = std::env::var_os("OUT_DIR").unwrap();
    build_tools::write_ffi(
        "shader_test_module",
        Path::new(&current_dir_os_string),
        &std::env::current_dir().unwrap().join("src/lib.rs"),
        true,
    )
    .unwrap();

    println!("Codegen finished.")
}
