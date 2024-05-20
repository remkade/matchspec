fn main() {
    if cfg!(target_os = "macos") {
        pyo3_build_config::add_extension_module_link_args();
    }
}
