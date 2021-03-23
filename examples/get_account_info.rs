#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    let pass = dotenv::var("OP_PASS").unwrap();
    //before using this crate, please make sure you have had set up the 1password-cli:
    //you need to login you 1password account.
    //for personal use it is usually command like `op singin my.1password.com [Mail Address] [Secret Key]`
    let op_cli = onepassword_cli::OpCLI::new_with_pass("my", &pass)
        .await
        .unwrap();
    let account = op_cli.get().account().run().await;
    println!("{:?}", account.unwrap());
}
