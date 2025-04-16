use std::future::Future;

async fn print(message: &str) {
    println!("{message}");
}

async fn credits(
    star: impl Future<Output = ()>,
    costar: impl Future<Output = ()>
) {
    println!("Starring:");
    star.await;
    println!("and");
    costar.await;
}

fn star(star_index: usize) -> impl Future<Output = ()> {
    let name = match star_index {
        0 => "Bonnie",
        1 => "Clyde",
        2 => "Mephistopheles",
        _ => "some minor character",
    };
    print(name)
}

#[tokio::main]
async fn main() {
    let bonnie = star(0);
    let clyde = star(1);

    credits(bonnie, clyde).await;
}
