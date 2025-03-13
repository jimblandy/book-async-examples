async fn print(message: &str) {
    println!("{message}");
}

#[tokio::main]
async fn main() {
    println!("Starring:");
    print("Bonnie");
}
