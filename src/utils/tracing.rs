use tracing::metadata::LevelFilter;

pub fn initialize_tracing() {
    // Configure the tracing subscriber
    tracing_subscriber::fmt().with_max_level(LevelFilter::INFO)
    .with_level(true)
    .with_target(true)
    .with_thread_names(true)
    .init();

}