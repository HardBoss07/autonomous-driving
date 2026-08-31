use autonomous_driving::run;
use macroquad::prelude::Conf;

fn window_conf() -> Conf {
    Conf {
        window_title: "Autonomous Driving Simulator".to_string(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    run().await;
}
