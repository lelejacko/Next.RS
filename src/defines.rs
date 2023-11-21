pub static DEFINES: &str = stringify! { // <=
$modules // <=

use engineioxide::service::NotFoundService;
use futures::{executor, future::ready};
use http_body::Body as HttpBody;
use hyper::{
    body::Bytes,
    header::HeaderValue,
    service::{make_service_fn, service_fn, Service},
    Body, Request as HyperRequest, Response as HyperResponse, Server,
};
use lazy_static::lazy_static;
use serde_json::Value;
use socketioxide::{adapter::LocalAdapter, service::SocketIoService, Socket, SocketIo};
use std::{
    collections::HashMap,
    convert::Infallible,
    fmt::Display,
    net::SocketAddr,
    str::FromStr,
    sync::{Arc, Mutex},
};

type SocketIOService = SocketIoService<LocalAdapter, NotFoundService>;

lazy_static! {
    static ref SOCKET_SERVICE: (Mutex<SocketIOService>, SocketIo) = {
        let (service, io) = SocketIo::builder().build_svc();
        (Mutex::new(service), io)
    };
    static ref SOCKETS: Mutex<HashMap<String, Arc<Socket<LocalAdapter>>>> =
        Mutex::new(HashMap::new());
}

/// HTTP request method
#[derive(Debug, Clone, PartialEq)]
pub enum ReqMethod {
    Get,
    Patch,
    Post,
    Put,
    Delete,
    Options,
    Head,
    Trace,
    Connect,
}

impl ReqMethod {
    pub fn from(string: &str) -> ReqMethod {
        match string {
            "GET" => ReqMethod::Get,
            "PATCH" => ReqMethod::Patch,
            "POST" => ReqMethod::Post,
            "PUT" => ReqMethod::Put,
            "DELETE" => ReqMethod::Delete,
            "OPTIONS" => ReqMethod::Options,
            "HEAD" => ReqMethod::Head,
            "TRACE" => ReqMethod::Trace,
            "CONNECT" => ReqMethod::Connect,
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
                ReqMethod::Options => "OPTIONS",
                ReqMethod::Head => "HEAD",
                ReqMethod::Trace => "TRACE",
                ReqMethod::Connect => "CONNECT",
            }
        )
    }
}

/// An HTTP Request
#[derive(Debug)]
pub struct Request {
    pub method: ReqMethod,
    pub path: String,
    pub body: Option<String>,
    pub headers: Vec<String>,

    /// If the path is matched against a dynamic route, the
    /// values of the dynamic fields are stored in this property.
    ///
    /// For example:
    /// If an API handler is created in `src/routes/api/books/__id.rs`
    /// all requests made to `/api/books/<id>` will be passed to this handler.
    /// The requests passed to this handler will have their `dyn_fields`
    /// property set to `Some({"id": "<id>"})`.
    pub dyn_fields: Option<HashMap<String, String>>,
}

impl Request {
    /// Get the request 'query parameters'
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

    /// Allow only the specified methods on the handler, returning
    /// `400 Bad request` if any other method is attempted.
    ///
    /// Example:
    /// ```rust
    /// pub async fn handler(req: Request) -> Result<Response, Response> {
    ///     req.allow_methods(vec![ReqMethod::Get])?;
    ///
    ///     Ok(Response::from_string(
    ///         200,
    ///         None,
    ///         Some(&format!("Hi from {}", req.path)),
    ///     ))
    /// }
    /// ```
    pub fn allow_methods(&self, methods: Vec<ReqMethod>) -> Result<(), Response> {
        if methods.contains(&self.method) {
            return Ok(());
        }

        Err(json_response!(400, {"message": "Method not allowed"}))
    }
}

/// An HTTP Response
#[derive(Debug)]
pub struct Response {
    pub code: u16,
    pub headers: Option<Vec<u8>>,
    pub body: Option<Vec<u8>>,
}

impl Response {
    /// Create a `Response` with the given `code`, `headers` and `body`
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

fn get_sio_service() -> SocketIOService {
    SOCKET_SERVICE.0.lock().unwrap().to_owned()
}

async fn handle_sio_request(req: HyperRequest<Body>) -> Result<HyperResponse<Body>, Infallible> {
    get_sio_service().call(req).await.map(|mut res| {
        // Response mapping
        let mut response = HyperResponse::builder();

        response = response.status(res.status());
        response = response.version(res.version());

        for header in res.headers() {
            response = response.header(header.0, header.1)
        }

        let body = Body::from(
            executor::block_on(res.data())
                .unwrap_or(Ok(Bytes::default()))
                .unwrap(),
        );
        response.body(body).unwrap()
    })
}

async fn handle(mut req: Request) -> Response {
    let req_path = req.path.clone();
    let clean_path = req_path.split("?").collect::<Vec<_>>()[0].trim_matches('/');

    let response = match clean_path {
        $handlers // <=
        _ => Err(json_response!(404, {"message": "Not found"})),
    };

    match response {
        Ok(r) => r,
        Err(r) => r,
    }
}

async fn handle_std_request(
    mut req: HyperRequest<Body>,
) -> Result<HyperResponse<Body>, Infallible> {
    let mut request = Request {
        method: ReqMethod::from(&req.method().to_string()),
        path: req.uri().to_string(),
        body: req.body_mut().data().await.map_or(None, |res| {
            res.map_or(None, |body| String::from_utf8(body.to_vec()).ok())
        }),
        headers: req
            .headers()
            .iter()
            .map(|h| format!("{}: {}", h.0, h.1.to_str().unwrap_or("")))
            .collect(),
        dyn_fields: None,
    };

    #[cfg(debug_assertions)]
    let method = request.method.clone();
    let path = request.path.clone();

    let response = handle(request).await;

    #[cfg(debug_assertions)]
    println!("{} {} → {}", method, path, response.code);

    let mut res = HyperResponse::builder().status(response.code);

    if let Some(hdrs) = response.headers {
        if let Some(headers) = String::from_utf8(hdrs).ok() {
            for header in headers.split("\n") {
                if let Some((key, value)) = header.split_once("=") {
                    res = res.header(key, value)
                }
            }
        }
    }

    let mut res_body = Body::empty();
    if let Some(b) = response.body {
        if let Some(body) = String::from_utf8(b).ok() {
            res_body = Body::from(body);
        }
    }

    Ok(res.body(res_body).unwrap())
}

async fn handle_request(req: HyperRequest<Body>) -> Result<HyperResponse<Body>, Infallible> {
    let result = match (req.uri().path(), req.headers().contains_key("Upgrade")) {
        (path, header) if header || ["/socket.io", "/ws"].iter().any(|p| path.starts_with(p)) => {
            handle_sio_request(req).await
        }
        _ => handle_std_request(req).await,
    };

    result.map(|mut res| {
        let headers = res.headers_mut();
        headers.insert("Access-Control-Allow-Origin", HeaderValue::from_static("*"));
        headers.insert(
            "Access-Control-Allow-Methods",
            HeaderValue::from_static("GET, POST, PATCH, PUT, DELETE, OPTIONS"),
        );
        headers.insert(
            "Access-Control-Allow-Headers",
            HeaderValue::from_static(
                "Origin, X-Requested-With, Content-Type, Accept, authorization",
            ),
        );

        res
    })
}

/// A web server (handling both `HTTP` and `socket.io` requests)
pub struct WebServer {
    pub address: SocketAddr,
}

impl WebServer {
    /// Create a new `WebServer` listening on the specified `port`.
    ///
    /// Currently only one instance should be created, because the `socket.io`
    /// service is `static`: this means that all server instances would share the
    /// same `socket.io` handling.
    pub fn new(port: u16) -> Self {
        let address = SocketAddr::from_str(&format!("0.0.0.0:{port}")).unwrap();

        WebServer { address }
    }

    /// Start the server.
    pub async fn start(&self) {
        let make_svc =
            make_service_fn(move |_| ready(Ok::<_, Infallible>(service_fn(handle_request))));
        let server = Server::bind(&self.address).serve(make_svc);

        #[cfg(debug_assertions)]
        println!("> Server running at http://{}", self.address);

        if let Err(e) = server.await {
            eprintln!("Server error: {e}")
        }
    }
}

/// A struct to access the `socket.io` methods
pub struct SocketIO;

impl SocketIO {
    // TODO: add namespace handling

    /// Create a given `namespace`, providing
    /// default auth and disconnection handling
    pub fn add_ns(namespace: &str) {
        SOCKET_SERVICE
            .1
            .ns(namespace, |socket, auth: Value| async move {
                #[cfg(debug_assertions)]
                println!("`Socket.IO` connected: {:?} {:?}", socket.ns(), socket.id);
                socket.emit("auth", auth).ok();

                socket.on_disconnect(|socket, reason| async move {
                    SOCKETS.lock().unwrap().remove(&socket.id.to_string());
                    #[cfg(debug_assertions)]
                    println!("Socket.IO disconnected: {} {}", socket.id, reason);
                });

                SOCKETS
                    .lock()
                    .unwrap()
                    .insert(socket.id.to_string(), socket);
            });
    }

    /// Emit the given `data` on the specified `namespace` `topic`
    pub fn emit(namespace: &str, topic: &str, data: Value) {
        #[cfg(debug_assertions)]
        println!("Emitting on namespace {namespace} topic {topic} → {data}");

        for socket in SOCKETS.lock().unwrap().values() {
            if socket.ns() == namespace {
                socket.emit(topic, data.clone()).ok();
            }
        }
    }
}

/// Create a json response
macro_rules! json_response {
    (
        $code:expr,
        $body:tt$(,)?
    ) => {
        Response::from_string(
            $code,
            Some("Content-Type=application/json"),
            Some(&serde_json::json!($body).to_string()),
        )
    };
}

pub(crate) use json_response;

}; // <=
