#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use next_rs::make_server;
pub mod routes {
    pub mod index_html {
        pub static HEADERS: &[u8] = b"Content-Type=text/html";
        pub static BODY: &[u8] = &[
            60,
            104,
            116,
            109,
            108,
            32,
            108,
            97,
            110,
            103,
            61,
            34,
            101,
            110,
            34,
            62,
            10,
            10,
            60,
            104,
            101,
            97,
            100,
            62,
            10,
            32,
            32,
            32,
            32,
            60,
            109,
            101,
            116,
            97,
            32,
            99,
            104,
            97,
            114,
            115,
            101,
            116,
            61,
            34,
            85,
            84,
            70,
            45,
            56,
            34,
            62,
            10,
            32,
            32,
            32,
            32,
            60,
            109,
            101,
            116,
            97,
            32,
            110,
            97,
            109,
            101,
            61,
            34,
            118,
            105,
            101,
            119,
            112,
            111,
            114,
            116,
            34,
            32,
            99,
            111,
            110,
            116,
            101,
            110,
            116,
            61,
            34,
            119,
            105,
            100,
            116,
            104,
            61,
            100,
            101,
            118,
            105,
            99,
            101,
            45,
            119,
            105,
            100,
            116,
            104,
            44,
            32,
            105,
            110,
            105,
            116,
            105,
            97,
            108,
            45,
            115,
            99,
            97,
            108,
            101,
            61,
            49,
            46,
            48,
            34,
            62,
            10,
            32,
            32,
            32,
            32,
            60,
            116,
            105,
            116,
            108,
            101,
            62,
            68,
            111,
            99,
            117,
            109,
            101,
            110,
            116,
            60,
            47,
            116,
            105,
            116,
            108,
            101,
            62,
            10,
            60,
            47,
            104,
            101,
            97,
            100,
            62,
            10,
            10,
            60,
            98,
            111,
            100,
            121,
            62,
            10,
            32,
            32,
            32,
            32,
            60,
            104,
            49,
            62,
            72,
            101,
            108,
            108,
            111,
            32,
            119,
            111,
            114,
            108,
            100,
            60,
            47,
            104,
            49,
            62,
            10,
            60,
            47,
            98,
            111,
            100,
            121,
            62,
            10,
            10,
            60,
            47,
            104,
            116,
            109,
            108,
            62,
        ];
    }
    pub mod api {
        pub mod v1 {
            pub mod __id {
                pub mod detail {
                    use crate::{Request, Response};
                    pub fn handler(req: Request) -> Response {
                        Response::from_string(
                            200,
                            None,
                            Some(
                                &{
                                    let res = ::alloc::fmt::format(
                                        format_args!("Hi from {0}", req.path),
                                    );
                                    res
                                },
                            ),
                        )
                    }
                }
            }
            pub mod test {
                use crate::{Request, Response};
                pub fn handler(req: Request) -> Response {
                    Response::from_string(
                        200,
                        None,
                        Some(
                            &{
                                let res = ::alloc::fmt::format(
                                    format_args!("Hi from {0}", req.path),
                                );
                                res
                            },
                        ),
                    )
                }
            }
            pub mod check {
                pub mod this {
                    use crate::{Request, Response};
                    pub fn handler(req: Request) -> Response {
                        Response::from_string(
                            200,
                            None,
                            Some(
                                &{
                                    let res = ::alloc::fmt::format(
                                        format_args!("Hi from {0}", req.path),
                                    );
                                    res
                                },
                            ),
                        )
                    }
                }
                pub mod __id {
                    use crate::{Request, Response};
                    pub fn handler(req: Request) -> Response {
                        Response::from_string(
                            200,
                            None,
                            Some(
                                &{
                                    let res = ::alloc::fmt::format(
                                        format_args!("Hi from {0}", req.path),
                                    );
                                    res
                                },
                            ),
                        )
                    }
                }
                pub mod r#mod {
                    use crate::{Request, Response};
                    pub fn handler(req: Request) -> Response {
                        Response::from_string(
                            200,
                            None,
                            Some(
                                &{
                                    let res = ::alloc::fmt::format(
                                        format_args!("Hi from {0}", req.path),
                                    );
                                    res
                                },
                            ),
                        )
                    }
                }
            }
            pub mod r#mod {
                use crate::{Request, Response};
                pub fn handler(req: Request) -> Response {
                    Response::from_string(
                        200,
                        None,
                        Some(
                            &{
                                let res = ::alloc::fmt::format(
                                    format_args!("Hi from {0}", req.path),
                                );
                                res
                            },
                        ),
                    )
                }
            }
        }
    }
}
use std::{
    collections::HashMap, fmt::Display, io::{BufRead, BufReader, Error, Read, Write},
    net::{TcpListener, TcpStream},
    thread::{spawn, JoinHandle},
};
pub enum ReqMethod {
    Get,
    Patch,
    Post,
    Put,
    Delete,
}
#[automatically_derived]
impl ::core::fmt::Debug for ReqMethod {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(
            f,
            match self {
                ReqMethod::Get => "Get",
                ReqMethod::Patch => "Patch",
                ReqMethod::Post => "Post",
                ReqMethod::Put => "Put",
                ReqMethod::Delete => "Delete",
            },
        )
    }
}
#[automatically_derived]
impl ::core::clone::Clone for ReqMethod {
    #[inline]
    fn clone(&self) -> ReqMethod {
        match self {
            ReqMethod::Get => ReqMethod::Get,
            ReqMethod::Patch => ReqMethod::Patch,
            ReqMethod::Post => ReqMethod::Post,
            ReqMethod::Put => ReqMethod::Put,
            ReqMethod::Delete => ReqMethod::Delete,
        }
    }
}
impl ReqMethod {
    pub fn from(string: &str) -> ReqMethod {
        match string {
            "GET" => ReqMethod::Get,
            "PATCH" => ReqMethod::Patch,
            "POST" => ReqMethod::Post,
            "PUT" => ReqMethod::Put,
            "DELETE" => ReqMethod::Delete,
            _ => ::core::panicking::panic_fmt(format_args!("Invalid method")),
        }
    }
}
impl Display for ReqMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(
            format_args!(
                "{0}",
                match self {
                    ReqMethod::Get => "GET",
                    ReqMethod::Patch => "PATCH",
                    ReqMethod::Post => "POST",
                    ReqMethod::Put => "PUT",
                    ReqMethod::Delete => "DELETE",
                },
            ),
        )
    }
}
pub struct Request {
    pub method: ReqMethod,
    pub path: String,
    pub body: Option<String>,
}
#[automatically_derived]
impl ::core::fmt::Debug for Request {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field3_finish(
            f,
            "Request",
            "method",
            &self.method,
            "path",
            &self.path,
            "body",
            &&self.body,
        )
    }
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
            query_params
                .insert(String::from(split_param[0]), String::from(split_param[1]));
        }
        Some(query_params)
    }
}
pub struct Response {
    pub code: u16,
    pub headers: Option<Vec<u8>>,
    pub body: Option<Vec<u8>>,
}
#[automatically_derived]
impl ::core::fmt::Debug for Response {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field3_finish(
            f,
            "Response",
            "code",
            &self.code,
            "headers",
            &self.headers,
            "body",
            &&self.body,
        )
    }
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
fn handle(req: Request) -> Response {
    let clean_path = req.path.split("?").collect::<Vec<_>>()[0].trim_matches('/');
    match clean_path {
        "" => {
            Response {
                code: 200,
                headers: Some(routes::index_html::HEADERS.to_vec()),
                body: Some(routes::index_html::BODY.to_vec()),
            }
        }
        p if p.starts_with("api/v1/") && p.contains("detail") && p.len() > 13 => {
            routes::api::v1::__id::detail::handler(req)
        }
        "api/v1/test" => routes::api::v1::test::handler(req),
        "api/v1/check/this" => routes::api::v1::check::this::handler(req),
        p if p.starts_with("api/v1/check/") && p.len() > 13 => {
            routes::api::v1::check::__id::handler(req)
        }
        "api/v1/check" => routes::api::v1::check::r#mod::handler(req),
        "api/v1" => routes::api::v1::r#mod::handler(req),
        _ => {
            Response {
                code: 404,
                headers: None,
                body: Some(b"Not found".to_vec()),
            }
        }
    }
}
struct ThreadPool {
    threads: Vec<Option<JoinHandle<()>>>,
}
impl ThreadPool {
    fn new() -> Self {
        let threads = ::alloc::vec::Vec::new();
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
        let listener = TcpListener::bind({
                let res = ::alloc::fmt::format(format_args!("0.0.0.0:{0}", port));
                res
            })
            .unwrap();
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
            {
                ::std::io::_print(
                    format_args!("{0:?} {1} => {2}\n", method, path, response.code),
                );
            };
            let mut res = {
                let res = ::alloc::fmt::format(
                    format_args!("HTTP/1.1 {0}", response.code),
                );
                res
            }
                .as_bytes()
                .to_vec();
            if let Some(mut headers) = response.headers {
                res.append(&mut headers);
            }
            if let Some(mut body) = response.body {
                res.append(&mut body);
            }
            stream.write_all(&res).unwrap();
        }
    }
    fn read_request(stream: &mut TcpStream) -> Option<Request> {
        let mut reader = BufReader::new(stream);
        let mut headers: Vec<String> = ::alloc::vec::Vec::new();
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(length) => {
                    if length < 3 {
                        break;
                    }
                }
                Err(e) => {
                    {
                        ::std::io::_print(format_args!("{0}\n", e));
                    };
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
            let content_length = l
                .split(":")
                .collect::<Vec<_>>()[1]
                .trim()
                .parse::<usize>()
                .unwrap();
            let mut bytes = ::alloc::vec::from_elem(0, content_length);
            reader.read_exact(&mut bytes).unwrap();
            content = String::from_utf8(bytes).unwrap_or(content);
        }
        let body = if content.is_empty() { None } else { Some(content) };
        Some(Request { method, path, body })
    }
}
fn main() {
    WebServer::start(8080);
}
