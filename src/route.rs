use {
    crate::mime_type::MimeType,
    std::{
        fs::{metadata, read_dir, read_to_string},
        process::exit,
    },
};

pub static ROUTES_PATH: &str = "src/routes";

#[derive(Debug)]
pub struct Route {
    path: String,
    children: Option<Vec<Route>>,
    static_body: Option<String>,
    mime_type: Option<MimeType>, // TODO: implement headers
}

impl Route {
    fn new(path: String) -> Self {
        let mut children: Option<Vec<Route>> = None;
        let mut static_body: Option<String> = None;
        let mut mime_type: Option<MimeType> = None;

        if metadata(&path).unwrap().is_dir() {
            children = Some(Self::get_children(&path));
        } else if !path.ends_with(".rs") {
            static_body = Some(
                read_to_string(&path)
                    .unwrap()
                    .replace("\n", "\\n")
                    .replace("\"", "\\\""),
            );

            let split_path = path.split(".").collect::<Vec<_>>();
            if split_path.len() > 1 {
                mime_type = MimeType::from(split_path[1]);
            }
        }

        Route {
            path,
            children,
            static_body,
            mime_type,
        }
    }

    pub fn base() -> Self {
        Self::check_is_dir(&ROUTES_PATH);
        Self::new(String::from(ROUTES_PATH))
    }

    fn check_is_dir(path: &str) {
        let invalid_path_err = || {
            println!("Path {path} is not a directory");
            exit(-1);
        };

        match metadata(path) {
            Ok(md) => {
                if !md.is_dir() {
                    invalid_path_err()
                }
            }
            _ => invalid_path_err(),
        }
    }

    fn get_children(base_path: &str) -> Vec<Self> {
        Self::check_is_dir(base_path);

        read_dir(base_path)
            .unwrap()
            .map(|e| {
                let entry = e.unwrap();
                let entry_path = entry.path();
                let path = String::from(entry_path.to_str().unwrap());

                Self::new(path)
            })
            .collect()
    }

    fn clean_path(&self) -> String {
        self.path
            .replace("src", "")
            .replace(".rs", "")
            .replace("/mod", "/r#mod")
            .trim_matches('/')
            .to_string()
    }

    fn route_path(&self) -> String {
        self.clean_path()
            .replace("routes", "")
            .replace("r#mod", "")
            .replace("index.html", "")
            .trim_matches('/')
            .to_string()
    }

    fn is_mod(&self) -> bool {
        self.path.ends_with(".rs")
            || (self.children.is_some()
                && self.children.as_ref().unwrap().iter().any(|c| c.is_mod()))
    }

    fn has_handler(&self) -> bool {
        metadata(&self.path).unwrap().is_file() && {
            let content = read_to_string(&self.path).unwrap();
            content.contains("pub fn handler(")
        }
    }

    fn is_api(&self) -> bool {
        self.is_mod() && self.has_handler()
    }

    fn is_static(&self) -> bool {
        self.static_body.is_some()
    }

    fn handler(&self) -> Option<String> {
        let mut handler = format!("\"{}\" => ", self.route_path(),);

        let mod_path = format!(
            "{}::",
            self.clean_path()
                .split("/")
                .collect::<Vec<_>>()
                .join("::")
                .replace(".", "_")
        );

        if self.is_api() {
            handler += &format!("{mod_path}handler(req)");
        } else if self.is_static() {
            handler += &format!("Response {{code: 200, body: Some(String::from({mod_path}BODY))}}");
        } else {
            return None;
        }

        Some(handler + ",")
    }

    fn mod_name(&self) -> String {
        let clean_path = self.clean_path();
        let split_path: Vec<_> = clean_path.split("/").collect();
        String::from(split_path[split_path.len() - 1]).replace(".", "_")
    }

    pub fn get_mod(&self, nesting: Option<usize>) -> String {
        let nest = if let Some(n) = nesting { n } else { 0 };
        let padding = vec!["    "; nest as usize].join("");
        let mut mod_str = format!("{}pub mod {}", padding, self.mod_name());

        if let Some(children) = &self.children {
            let sub_mods = &children
                .iter()
                .map(|c| c.get_mod(Some(nest + 1)))
                .collect::<Vec<_>>()
                .join("\n");
            mod_str += &format!(" {{\n{}\n{padding}}}", sub_mods);
        } else if self.is_static() {
            mod_str += &format!(
                " {{\n{padding}    pub static BODY: &str = \"{}\";\n{padding}}}",
                self.static_body.clone().unwrap()
            )
        } else {
            mod_str += ";";
        }

        mod_str
    }

    pub fn get_handlers(&self) -> Vec<String> {
        let mut handlers: Vec<String> = vec![];

        if let Some(children) = &self.children {
            handlers.append(
                &mut children
                    .iter()
                    .flat_map(|c| c.get_handlers())
                    .collect::<Vec<_>>(),
            );
        } else if let Some(handler) = self.handler() {
            handlers.push(handler);
        }

        handlers
    }
}
