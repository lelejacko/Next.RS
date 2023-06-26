pub static DEFINES: &str = stringify! { // <=
$modules // <=

use std::{
    collections::HashMap,
    fmt::Display,
    io::{BufRead, BufReader, Error, Read, Write},
    net::{TcpListener, TcpStream},
    thread::{spawn, JoinHandle},
};

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
    pub dyn_fields: Option<HashMap<String, String>>,
}

impl Request {
    pub fn query_params(&self) -> Option<HashMap<String, String>> {
        if !self.path.contains("?") {
            return None;
        }

        let (_, params_string) = self.path.rsplit_once("?").unwrap();
        let mut query_params: HashMap<String, String> = HashMap::new();

        for param in params_string.split("&") {
            if !param.contains("=") {
                continue;
            }

            let split_param: Vec<_> = param.split("=").collect();
            query_params.insert(String::from(split_param[0]), String::from(split_param[1]));
        }

        Some(query_params)
    }
}

#[derive(Debug)]
pub struct Response {
    pub code: u16,
    pub headers: Option<Vec<u8>>,
    pub body: Option<Vec<u8>>,
}

impl Response {
    pub fn from_string(code: u16, headers: Option<&str>, body: Option<&str>) -> Self {
        Response {
            code,
            headers: headers.map(|h| h.as_bytes().to_vec()),
            body: body.map(|b| b.as_bytes().to_vec()),
        }
    }
}

fn get_dynamic_fields(path: &str, dynamic_route: &str) -> Option<HashMap<String, String>> {
    let mut dyn_fields: HashMap<String, String> = HashMap::new();

    for (i, s) in dynamic_route.split("__").enumerate() {
        let (field, matcher) = if i == 0 {
            ("", s)
        } else {
            s.split_once("/").unwrap_or((s, ""))
        };

        if !path.contains(matcher) {
            return None;
        }

        if field.is_empty() {
            continue;
        }

        let mut value = if matcher.is_empty() {
            path
        } else {
            path.split_once(matcher).unwrap_or(("", "")).0
        };

        if value.contains("/") {
            value = value.trim_matches('/').rsplit_once("/").unwrap().1;
        }

        if value.is_empty() {
            return None;
        }

        dyn_fields.insert(String::from(field), String::from(value));
    }

    Some(dyn_fields)
}

fn matches_dynamic_route(path: &str, dynamic_route: &str, req: &mut Request) -> bool {
    req.dyn_fields = get_dynamic_fields(path, dynamic_route);
    req.dyn_fields.is_some()
}

fn handle(mut req: Request) -> Response {
    let req_path = req.path.clone();
    let clean_path = req_path.split("?").collect::<Vec<_>>()[0].trim_matches('/');

    match clean_path {
        $handlers // <=
        _ => Response {
            code: 404,
            headers: None,
            body: Some(b"Not found".to_vec()),
        },
    }
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

        if let Some(mut request) = Self::read_request(&mut stream) {
            #[cfg(debug_assertions)]
            let method = request.method.clone();

            #[cfg(debug_assertions)]
            let path = request.path.clone();

            let response = handle(request);

            #[cfg(debug_assertions)]
            println!("{:?} {} => {}", method, path, response.code);

            let mut res = format!("HTTP/1.1 {}\r\n", response.code)
                .as_bytes()
                .to_vec();

            if let Some(mut headers) = response.headers {
                res.append(&mut headers);
            }

            res.append(&mut b"\r\n\r\n".to_vec());

            if let Some(mut body) = response.body {
                res.append(&mut body);
            }

            stream.write_all(&res).unwrap();
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

        Some(Request {
            method,
            path,
            body,
            dyn_fields: None,
        })
    }
}
}; // <=
