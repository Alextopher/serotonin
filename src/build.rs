fn main() {
    std::fs::read_dir("src/libraries").unwrap().into_iter().for_each(|lib| 
        println!("cargo:rerun-if-changed=src/libraries/{}", lib.unwrap().file_name().to_str().unwrap())
    )
}
