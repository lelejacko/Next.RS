# NextRS
NextRS is a web backend framework.

### Features:
- **Filesystem based routes**. All files under `src/*/routes` folder are exposed:
    - as API if the file is a `.rs` module exporting a `pub fn handler` that takes a `Request` type parameter and returns a `Response` object
    - as a static content in other cases
- **Dynamic routes**. If a file or a directory name starts with `"__"` it is used as a wildcard in routes matching
- **Query params parsing**. Query parameters can be accessed as `HashMap` object with through the `Request.query_params()` method

### TODOs
- [ ] Improve documentation
