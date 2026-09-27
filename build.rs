fn main() {
    println!("cargo:rerun-if-env-changed=FEROS_BUILD_COMMIT");
    println!("cargo:rerun-if-env-changed=FEROS_BUILD_TIMESTAMP");
}
