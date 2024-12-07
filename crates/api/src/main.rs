#[rocket::main]
async fn main() -> anyhow::Result<()> {
    api::run_server().await
}
