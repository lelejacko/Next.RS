use next_rs::make_server;
use serde_json::json;

make_server!();

#[tokio::main]
async fn main() {
    let server = WebServer::new(8080);

    SocketIO::add_ns("/");
    std::thread::spawn(move || {
        let mut i = 0;

        loop {
            i += 1;
            SocketIO::emit("/", "message", json!(format!("Counter: {i}")));
            std::thread::sleep(std::time::Duration::from_secs(2))
        }
    });

    server.start().await;
}
