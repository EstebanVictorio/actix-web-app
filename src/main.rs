use actix_web_app::MessageApp;
// NOTE: last page: 70
// NOTE: next page: 71
fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG","actix_web=info");
    env_logger::init();
    let app = MessageApp::new(8080);
    app.run()
}
