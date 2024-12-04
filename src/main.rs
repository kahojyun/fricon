use fricon::main as lib_main;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    lib_main().await
}
