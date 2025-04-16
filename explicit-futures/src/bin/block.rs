async fn print(message: &str) {
    println!("{message}");
}

fn main() -> std::io::Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;

    let bonnie = print("Bonnie");
    let clyde = print("Clyde");

    let credits = async {
        bonnie.await;
        println!("and");
        clyde.await;
    };
    
    println!("Starring:");
    runtime.block_on(credits);

    Ok(())
}
