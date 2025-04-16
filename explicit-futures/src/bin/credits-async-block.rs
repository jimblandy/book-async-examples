use std::future::Future;

async fn print(message: &str) {
    println!("{message}");
}

fn copying_print(message: &str) -> impl Future<Output = ()> {
    let owned_message = message.to_string();
    async move {
        println!("{owned_message}");
    }
}

async fn bad_print_lifetime(star: &str, costar: &str) {
    let future = {
        let credits = format!("{star} and {costar}");
        copying_print(&credits)
    };
    future.await;
}

fn credits(
    star: impl Future<Output = ()>,
    costar: impl Future<Output = ()>
) -> impl Future<Output = ()> {
    async {
        println!("Starring:");
        star.await;
        println!("and");
        costar.await;
    }
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
