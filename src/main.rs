extern crate librebbs;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    librebbs::run().await
}
