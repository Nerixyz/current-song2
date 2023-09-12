use mpris_dbus::tracks::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let mut listener = listen("org.mpris.MediaPlayer2.spotify").await.unwrap();
    println!("Waiting for events...");
    while let Some(state) = listener.recv().await {
        println!("{state:#?}");
    }
}
