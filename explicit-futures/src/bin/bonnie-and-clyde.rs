async fn print(message: &str) {
    println!("{message}");
}

#[tokio::main]
async fn main() {
    let bonnie = print("Bonnie");
    let clyde = print("Clyde");

    println!("Starring:");
    clyde.await;
    println!("and");
    bonnie.await;
}
