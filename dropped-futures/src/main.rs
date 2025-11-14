async fn print(message: &str) {
    println!("{message}");
}

#[allow(unused_must_use, reason = "deliberate anti-example")]
#[tokio::main]
async fn main() {
    println!("Starring:");
    print("Bonnie");
}
