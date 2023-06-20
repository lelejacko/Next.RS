use web_server::make_server;

make_server!("src/api");

fn main() {
    WebServer::start(8080);
}
