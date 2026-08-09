mod app;
mod shell;

#[tokio::main]
async fn main() {

    let app = app::Application::new();

    app.run().await;
}
