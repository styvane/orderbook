fn main() {
    tonic_build::configure()
        .type_attribute(
            "orderbook.Book",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .compile(&["proto/orderbook.proto"], &["proto"])
        .unwrap_or_else(|e| panic!("failed to compile protos {e:?}"));
}
