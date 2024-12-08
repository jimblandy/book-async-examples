/*
async fn print(message: &str) {
    println!("{message}");
}
*/

fn print<'a>(message: &'a str) -> impl std::future::Future<Output = ()> + 'a {
    async move {
        println!("{message}");
    }
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
