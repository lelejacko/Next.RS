pub static DEFINES: &str = stringify! { // <=
$modules // <=

use engineioxide::service::NotFoundService;
use futures::{executor, StreamExt};
use http_body_util::{BodyExt, BodyStream, Full};
use hyper::{
    body::{Bytes, Incoming},
    header::{HeaderValue, CONTENT_TYPE},
    server::conn::http1,
    service::{service_fn, Service},
    Request as HyperRequest, Response as HyperResponse,
};
use hyper_util::rt::TokioIo;
use lazy_static::lazy_static;
use multer::Multipart;
use regex::Regex;
use serde_json::Value;
use socketioxide::{
    adapter::LocalAdapter,
    extract::{Data, SocketRef},
    service::SocketIoService,
    socket::DisconnectReason,
    SocketIo,
};
use std::{
    collections::HashMap,
    convert::Infallible,
    fmt::Display,
    fs::{metadata, File},
    io::Write,
    net::SocketAddr,
    path::Path,
    str::FromStr,
    sync::Mutex,
};
use tokio::net::TcpListener;

type SocketIOService = SocketIoService<NotFoundService, LocalAdapter>;

lazy_static! {
    static ref SOCKET_SERVICE: (Mutex<SocketIOService>, SocketIo) = {
        let (service, io) = SocketIo::new_svc();
        (Mutex::new(service), io)
    };
    static ref SOCKETS: Mutex<HashMap<String, SocketRef>> = Mutex::new(HashMap::new());
    static ref DYN_FIELDS_REGEX: Regex = Regex::new(r"__(?P<field>[\w\-_]+)").unwrap();
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
pub struct Request<'a> {
    pub method: ReqMethod,
    pub path: String,
    pub body: Option<String>,
    pub multipart_body: Option<Multipart<'a>>,
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

impl<'a> Request<'_> {
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

        if self.method == ReqMethod::Options {
            return Err(json_response!(200, ""));
        }

        Err(json_response!(400, {"message": "Method not allowed"}))
    }

    /// Processes the multipart body of the request,
    /// uploading the files to the specified `dest`ination.
    /// The resulting Map contains the fields values and the
    /// path of the uploaded files.
    pub async fn process_upload<P>(self, dest: P) -> Result<HashMap<String, String>, Response>
    where
        P: AsRef<Path>,
    {
        if self.multipart_body.is_none() {
            return Err(json_response!(400, {"message": "Bad request"}));
        }

        if !metadata(dest.as_ref()).map_or(false, |m| m.is_dir()) {
            panic!("Speicified destination is not a directory");
        }

        let mut fields = HashMap::<String, String>::new();
        let mut multipart_body = self.multipart_body.unwrap();

        while let Some(mut field) = multipart_body.next_field().await.unwrap() {
            let name = field.name().map(|n| n.to_string());
            let file_name = field.file_name();

            let value = if let Some(fname) = file_name {
                let path = dest.as_ref().join(fname);
                let mut file = File::create(&path).unwrap();

                while let Some(chunk) = field.chunk().await.unwrap() {
                    file.write(&chunk).ok();
                }

                path.to_str().unwrap().to_string()
            } else {
                field.text().await.unwrap_or("".to_string())
            };

            fields.insert(name.unwrap(), value);
        }

        Ok(fields)
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
    let regex_src = DYN_FIELDS_REGEX
        .replace_all(&("^".to_string() + dynamic_route + "$"), "(?P<$field>[\\w\\-_]+)")
        .to_string();

    let regex = Regex::new(&regex_src).unwrap();
    let path_match = regex.captures(path);

    if path_match.is_none() {
        return None;
    }

    let path_match = path_match.unwrap();

    Some(
        regex
            .capture_names()
            .skip(1)
            .map(|cn| {
                let name = cn.unwrap();
                (
                    name.to_string(),
                    path_match.name(name).unwrap().as_str().to_string(),
                )
            })
            .collect(),
    )
}

fn matches_dynamic_route(path: &str, dynamic_route: &str, req: &mut Request) -> bool {
    req.dyn_fields = get_dynamic_fields(path, dynamic_route);
    req.dyn_fields.is_some()
}

fn get_sio_service() -> SocketIOService {
    SOCKET_SERVICE.0.lock().unwrap().clone()
}

async fn handle_sio_request(
    req: HyperRequest<Incoming>,
) -> Result<HyperResponse<Full<Bytes>>, Infallible> {
    get_sio_service().call(req).await.map(|res| {
        // Response mapping
        let mut response = HyperResponse::builder()
            .status(res.status())
            .version(res.version());

        for header in res.headers() {
            response = response.header(header.0, header.1)
        }

        let body = Full::new(executor::block_on(async move {
            res.collect()
                .await
                .ok()
                .map_or(Bytes::new(), |b| b.to_bytes())
        }));

        response.body(body).unwrap()
    })
}

async fn handle(mut req: Request<'_>) -> Response {
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

async fn map_request<'a>(req: HyperRequest<Incoming>) -> Request<'a> {
    let mut request = Request {
        method: ReqMethod::from(&req.method().to_string()),
        path: req.uri().to_string(),
        body: None,
        headers: req
            .headers()
            .iter()
            .map(|h| format!("{}: {}", h.0, h.1.to_str().unwrap_or("")))
            .collect(),
        dyn_fields: None,
        multipart_body: None,
    };

    let multipart_boundary = req
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|ct| ct.to_str().ok())
        .and_then(|ct| multer::parse_boundary(ct).ok());

    if let Some(boundary) = multipart_boundary {
        let body_stream = BodyStream::new(req.into_body()).filter_map(|result| async move {
            result.map(|frame| frame.into_data().ok()).transpose()
        });

        request.multipart_body = Some(Multipart::new(body_stream, boundary));
    } else {
        request.body = req.collect().await.ok().map_or(None, |b| {
            String::from_utf8(b.to_bytes().iter().cloned().collect()).ok()
        });
    }

    request
}

async fn handle_std_request(
    req: HyperRequest<Incoming>,
) -> Result<HyperResponse<Full<Bytes>>, Infallible> {
    let request = map_request(req).await;

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

    let res_body = if let Some(body) = response.body {
        Full::new(Bytes::from(body))
    } else {
        Full::new(Bytes::default())
    };

    Ok(res.body(res_body).unwrap())
}

async fn handle_request(
    req: HyperRequest<Incoming>,
) -> Result<HyperResponse<Full<Bytes>>, Infallible> {
    let result = match (req.uri().path(), req.headers().contains_key("Upgrade")) {
        (path, header) if header || ["/socket.io", "/ws"].iter().any(|p| path.starts_with(p)) => {
            handle_sio_request(req).await
        }
        _ => handle_std_request(req).await,
    };

    // TODO: let set custom headers
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
        let listener = TcpListener::bind(self.address).await.unwrap();
        let service = service_fn(handle_request);

        #[cfg(debug_assertions)]
        println!("> Server running at http://{}", self.address);

        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let io = TokioIo::new(stream);

            tokio::task::spawn(async move {
                if let Err(e) = http1::Builder::new()
                    .serve_connection(io, service.clone())
                    .with_upgrades()
                    .await
                {
                    #[cfg(debug_assertions)]
                    eprintln!("Server error: {e}")
                }
            });
        }
    }
}

/// A struct to access the `socket.io` methods
pub struct SocketIO;

impl SocketIO {
    // TODO: add namespace handling

    pub fn has_connections() -> bool {
        !SOCKETS.lock().unwrap().is_empty()
    }

    /// Create a given `namespace`, providing
    /// default auth and disconnection handling
    pub fn add_ns(namespace: &str) {
        let namespace = namespace.to_string();
        SOCKET_SERVICE.1.ns(
            namespace,
            |socket: SocketRef, Data(data): Data<Value>| async move {
                #[cfg(debug_assertions)]
                println!("`Socket.IO` connected: {:?} {:?}", socket.ns(), socket.id);
                socket.emit("auth", &data).ok();

                socket.on_disconnect(|socket: SocketRef, reason: DisconnectReason| async move {
                    SOCKETS.lock().unwrap().remove(&socket.id.to_string());
                    #[cfg(debug_assertions)]
                    println!("Socket.IO disconnected: {} {}", socket.id, reason);
                });

                SOCKETS
                    .lock()
                    .unwrap()
                    .insert(socket.id.to_string(), socket);
            },
        );
    }

    /// Emit the given `data` on the specified `namespace` `topic`
    pub fn emit(namespace: &str, topic: &str, data: Value) {
        #[cfg(debug_assertions)]
        println!("Emitting on namespace {namespace} topic {topic} → {data}");

        let topic = topic.to_string();
        for socket in SOCKETS.lock().unwrap().values() {
            if socket.ns() == namespace {
                socket.emit(topic.clone(), &data).ok();
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
