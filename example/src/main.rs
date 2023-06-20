use next_rs::make_server;

make_server!("src/api");

fn main() {
    WebServer::start(8080);
}
