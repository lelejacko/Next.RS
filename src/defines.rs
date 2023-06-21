pub static DEFINES: &str = stringify! {
$modules

use std::{
    fmt::Display,
    io::{BufRead, BufReader, Error, Read, Write},
    net::{TcpListener, TcpStream},
    thread::{spawn, JoinHandle},
};

fn handle(req: Request) -> Response {
    let clean_path = req.path.split("?").collect::<Vec<_>>()[0].trim_matches('/');

    match clean_path {
        $handles
        _ => Response {
            code: 404,
            body: Some(String::from("Not found")),
        },
    }
}

#[derive(Debug, Clone)]
pub enum ReqMethod {
    Get,
    Patch,
    Post,
    Put,
    Delete,
}

impl ReqMethod {
    pub fn from(string: &str) -> ReqMethod {
        match string {
            "GET" => ReqMethod::Get,
            "PATCH" => ReqMethod::Patch,
            "POST" => ReqMethod::Post,
            "PUT" => ReqMethod::Put,
            "DELETE" => ReqMethod::Delete,
            _ => panic!("Invalid method"),
        }
    }
}

impl Display for ReqMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ReqMethod::Get => "GET",
                ReqMethod::Patch => "PATCH",
                ReqMethod::Post => "POST",
                ReqMethod::Put => "PUT",
                ReqMethod::Delete => "DELETE",
            }
        )
    }
}

#[derive(Debug)]
pub struct Request {
    pub method: ReqMethod,
    pub path: String,
    pub body: Option<String>,
}

#[derive(Debug)]
pub struct Response {
    pub code: u16,
    pub body: Option<String>,
}

struct ThreadPool {
    threads: Vec<Option<JoinHandle<()>>>,
}

impl ThreadPool {
    fn new() -> Self {
        let threads = vec![];
        ThreadPool { threads }
    }

    fn add<F>(&mut self, job: F)
    where
        F: FnOnce() -> (),
        F: Send + 'static,
    {
        self.threads.push(Some(spawn(job)));
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for thread in &mut self.threads {
            if let Some(t) = thread.take() {
                t.join().unwrap();
            }
        }
    }
}

struct WebServer;

impl WebServer {
    pub fn start(port: u16) {
        let listener = TcpListener::bind(format!("0.0.0.0:{port}")).unwrap();

        let mut pool = ThreadPool::new();

        for connection in listener.incoming() {
            pool.add(|| Self::handle_connection(connection));
        }
    }

    fn handle_connection(connection: Result<TcpStream, Error>) {
        if connection.is_err() {
            return;
        }

        let mut stream = connection.unwrap();

        if let Some(request) = Self::read_request(&mut stream) {
            #[cfg(debug_assertions)]
            let method = request.method.clone();

            #[cfg(debug_assertions)]
            let path = request.path.clone();

            let response = handle(request);

            #[cfg(debug_assertions)]
            println!("{:?} {} => {}", method, path, response.code);

            let res_string = format!(
                "HTTP/1.1 {} \r\n\r\n{}",
                response.code,
                if let Some(content) = response.body {
                    content
                } else {
                    String::new()
                }
            );

            stream.write_all(res_string.as_bytes()).unwrap();
        }
    }

    fn read_request(stream: &mut TcpStream) -> Option<Request> {
        let mut reader = BufReader::new(stream);

        let mut headers: Vec<String> = vec![];
        loop {
            let mut line = String::new();

            match reader.read_line(&mut line) {
                Ok(length) => {
                    if length < 3 {
                        break;
                    }
                }
                Err(e) => {
                    println!("{e}");
                    return None;
                }
            }

            headers.push(line);
        }

        let first_header: Vec<_> = headers[0].split(" ").collect();
        let method = ReqMethod::from(first_header[0]);
        let path = String::from(first_header[1]);

        let mut content = String::new();
        if let Some(l) = headers.iter().find(|l| l.starts_with("Content-Length")) {
            let content_length = l.split(":").collect::<Vec<_>>()[1]
                .trim()
                .parse::<usize>()
                .unwrap();

            let mut bytes = vec![0; content_length];
            reader.read_exact(&mut bytes).unwrap();

            content = String::from_utf8(bytes).unwrap_or(content);
        }

        let body = if content.is_empty() {
            None
        } else {
            Some(content)
        };

        Some(Request { method, path, body })
    }
}
};
