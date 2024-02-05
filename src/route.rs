use super::mime_type::MimeType;
use lazy_static::lazy_static;
use std::{
    cmp::Ordering,
    fs::{metadata, read_dir, read_to_string},
    process::{exit, Command},
};

static ROUTES_DIR: &str = "routes";

lazy_static! {
    static ref ACTUAL_ROUTES_PATH: String = String::from_utf8(
        Command::new("sh")
            .args(["-c", &format!("find . | grep -oP \".*routes$\"")])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_string();
    static ref CALL_SITE_PATH: String = String::from_utf8(
        Command::new("sh")
            .args([
                "-c",
                "grep -Rw . -e \"make_server\\!\" | grep -oP \".*(?<=.rs)\" | grep -v target"
            ])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .split_once('\n')
    .unwrap()
    .0
    .rsplit_once('/')
    .unwrap()
    .0
    .trim()
    .to_string();
}

#[derive(Debug)]
pub struct Route {
    path: String,
    children: Option<Vec<Route>>,
    static_body: Option<String>,
    mime_type: Option<MimeType>,
}

impl Route {
    fn new(path: String) -> Self {
        let mut children: Option<Vec<Route>> = None;
        let mut static_body: Option<String> = None;
        let mut mime_type: Option<MimeType> = None;

        if metadata(&path).unwrap().is_dir() {
            children = Some(Self::get_children(&path));
        } else if !path.ends_with(".rs") {
            println!("Path: {path}\nCall site path: {}", &*CALL_SITE_PATH);

            let relative_path = if path.contains(&*CALL_SITE_PATH) {
                path.replace(&*CALL_SITE_PATH, "")
                    .trim_start_matches('/')
                    .to_string()
            } else {
                panic!("'routes' folder and the 'make_server!' macro call must be in the same directory if there are static files.");
            };

            static_body = Some(format!("include_bytes!(\"{relative_path}\")",));
            mime_type = path
                .rsplit_once('.')
                .map_or(None, |(_, ext)| MimeType::from(ext));
        }

        Route {
            path,
            children,
            static_body,
            mime_type,
        }
    }

    pub fn base() -> Self {
        Self::check_is_dir(&*ACTUAL_ROUTES_PATH);
        Self::new(ACTUAL_ROUTES_PATH.clone())
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
        let path = String::from(ROUTES_DIR) + self.path.split(ROUTES_DIR).collect::<Vec<_>>()[1];

        path.replace(".rs", "")
            .replace("/mod", "/r#mod")
            .replace("/super", "/_super")
            .trim_matches('/')
            .to_string()
    }

    fn route_matcher(&self) -> String {
        let clean_path = format!(
            "\"{}\"",
            self.clean_path()
                .replacen(ROUTES_DIR, "", 1)
                .replace("r#mod", "")
                .replace("/_super", "/super")
                .replace("index.html", "")
                .trim_matches('/')
                .to_string()
        );

        if !clean_path.contains("/__") {
            return clean_path;
        }

        format!("path if matches_dynamic_route(path, {clean_path}, &mut req)")
    }

    fn is_mod(&self) -> bool {
        self.path.ends_with(".rs")
            || (self.children.is_some()
                && self.children.as_ref().unwrap().iter().any(|c| c.is_mod()))
    }

    fn has_handler(&self) -> bool {
        metadata(&self.path).unwrap().is_file() && {
            let content = read_to_string(&self.path).unwrap();
            content.contains("pub async fn handler(")
        }
    }

    fn is_api(&self) -> bool {
        self.is_mod() && self.has_handler()
    }

    fn is_static(&self) -> bool {
        self.static_body.is_some()
    }

    fn clean_name(&self, name: &str) -> String {
        name.replace(".", "_")
            .replace("-", "_")
            .replace(",", "_")
            .replace(";", "_")
    }

    fn handler(&self) -> Option<String> {
        let mut handler = format!("{} => ", self.route_matcher());

        let mod_path = format!(
            "{}::",
            self.clean_name(&self.clean_path().split("/").collect::<Vec<_>>().join("::"))
        );

        if self.is_api() {
            handler += &format!("{mod_path}handler(req).await");
        } else if self.is_static() {
            handler += &format!(
                "Ok(Response {{
                    code: 200, 
                    headers: Some({mod_path}HEADERS.to_vec()), 
                    body: Some({mod_path}BODY.to_vec())
                }})"
            );
        } else {
            return None;
        }

        Some(handler + ",")
    }

    fn mod_path(&self) -> String {
        self.path
            .rsplit_once("/")
            .unwrap_or(("", &self.path))
            .1
            .to_string()
    }

    fn mod_name(&self) -> String {
        let clean_path = self.clean_path();
        self.clean_name(clean_path.rsplit_once("/").unwrap_or(("", &clean_path)).1)
    }

    pub fn get_mod(&self) -> String {
        let mut mod_str = format!(
            "#[path = \"{}\"]\npub mod {}",
            self.mod_path(),
            self.mod_name(),
        );

        if let Some(children) = &self.children {
            let sub_mods = &children
                .iter()
                .map(|c| c.get_mod())
                .collect::<Vec<_>>()
                .join("\n");
            mod_str += &format!(" {{{}}}", sub_mods);
        } else if self.is_static() {
            mod_str += &format!(
                "{{
                    pub static HEADERS: &'static [u8] = b\"{}\";
                    pub static BODY: &'static [u8] = {};
                }}",
                if let Some(mime_type) = &self.mime_type {
                    format!("Content-Type={}", mime_type.get())
                } else {
                    String::new()
                },
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

        handlers.sort_by(|h1, h2| {
            let h1_dyn = h1.contains("matches_dynamic_route");
            let h2_dyn = h2.contains("matches_dynamic_route");

            if h1_dyn == h2_dyn {
                Ordering::Equal
            } else if h1_dyn && !h2_dyn {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        });

        handlers
    }
}
