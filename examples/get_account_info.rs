use onepassword_cli::{OpCLI, SecondCmd};

#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    let pass = dotenv::var("OP_PASS").unwrap();
    let op_cli = OpCLI::new_with_pass("my", &pass).await.unwrap();
    let account = op_cli.get().account().run().await;
    println!("{:?}", account.unwrap());
}
