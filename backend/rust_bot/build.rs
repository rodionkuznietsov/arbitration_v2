fn main() {
    prost_build::compile_protos(
        &["src/exchanges/mexc/orderbook.proto"],
        &["src/exchanges/mexc"]
    ).expect("Failed to compile proto");

    println!("cargo:warning=OUT_DIR = {}", std::env::var("OUT_DIR").unwrap());
}