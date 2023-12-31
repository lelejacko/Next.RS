# NextRS
NextRS is a web backend framework.

### Features:
- **Filesystem based routes**. All files under `src/**/routes` folder are exposed:
    - as API if the file is a `.rs` module exporting a `pub async fn handler` that takes a `Request` type parameter and returns a `Result<Response, Response>` object
    - as a static content in other cases
- **Dynamic routes**. If a file or a directory under the `routes` folder starts with `"__"` it is used as a wildcard in routes matching (see the `Request.dyn_fields` property)
- **Query params parsing**. Query parameters can be accessed as an `HashMap` object with through the `Request.query_params()` method
- `socket.io` handling on the same `HTTP` port.

### To install:
Add the following dependencies to your `Cargo.toml`:
```toml
next_rs = { git = "https://github.com/lelejacko/Next.RS.git" }
engineioxide = "0.6.0"
futures = "0.3.29"
http-body = "0.4.5"
hyper = { version = "0.14.27", features = ["full"] }
lazy_static = "1.4.0"
serde_json = "^1.0.107"
socketioxide = "0.6.0"
tokio = { version = "^1.33.0", features = ["macros", "rt-multi-thread"] }
```

### Example
- File: `src/main.rs`
    ```rust
    use next_rs::make_server;
    use serde_json::json;

    // ↓ This creates all the components definitions
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
    ```

- File: `src/**/routes/api/test.rs` (served at `/api/test`)
    ```rust

    use crate::{ReqMethod, Request, Response};

    pub async fn handler(req: Request) -> Result<Response, Response> {
        req.allow_methods(vec![ReqMethod::Get])?;

        Ok(Response::from_string(
            200,
            None,
            Some(&format!("Hi from {}", req.path)),
        ))
    }
    ```

    Response at `/api/test`:
    ```json
    // 200 Ok

    "Hi from /api/test"
    ```

- File: `src/**/routes/index.html` (served at `/`)
    ```html
    <html lang="en">

    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Home</title>
    </head>

    <body>
        <h1>Hello world!</h1>

        <script src="https://cdn.socket.io/4.5.4/socket.io.min.js"></script>
        <script>
            function setMessage(msg) {
                document.querySelector("h1").textContent = msg;
            }

            const socket = io("http://127.0.0.1:8080");

            socket.on("connect", () => console.log("Socket.io connected!"));
            socket.on("error", (e) =>  console.log(`Error: ${e.code} => ${e.message} - ${e.toString()}`));
            socket.on("message", setMessage);
        </script>
    </body>

    </html>
    ```

    Response at `/`:
    ![Index.html](resources/index_response.gif)