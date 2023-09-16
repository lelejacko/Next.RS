use next_rs::make_server;

make_server!();

fn main() {
    WebServer::new(8080).start();
}
