use crate::engine::Engine;

pub async fn run_basic_window() {
    let engine = Engine::new().await;
    log::info!("Starting basic window example");
    engine.run();
}