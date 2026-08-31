pub mod app;
pub mod core;
pub mod render;
pub mod trainer;

use app::App;

pub async fn run() {
    let mut app = App::new().await;
    app.run_loop().await;
}
