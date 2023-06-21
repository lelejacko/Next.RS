use next_rs::make_server;

make_server!();

fn main() {
    WebServer::start(8080);
}
