mod defines;

use {
    defines::DEFINES as ext_defines,
    proc_macro::TokenStream,
    std::{
        fs::{metadata, read_dir, read_to_string},
        process::exit,
    },
};

#[derive(Debug, Clone)]
struct Route {
    path: String,
    children: Option<Vec<Route>>,
    nesting: usize,
}

impl Route {
    fn new(path: String, nesting: usize) -> Self {
        let mut children: Option<Vec<Route>> = None;
        if metadata(&path).unwrap().is_dir() {
            children = Some(Self::get_children(&path, nesting + 1));
        }

        Route {
            path,
            children,
            nesting,
        }
    }

    fn base(path: String) -> Self {
        Self::check_is_dir(&path);
        Self::new(path, 0)
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

    fn get_children(base_path: &str, base_nesting: usize) -> Vec<Self> {
        Self::check_is_dir(base_path);

        let nesting = base_nesting + 1;

        read_dir(base_path)
            .unwrap()
            .map(|e| {
                let entry = e.unwrap();
                let entry_path = entry.path();
                let path = String::from(entry_path.to_str().unwrap());

                Self::new(path, nesting)
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
        self.clean_path().replace("/r#mod", "")
    }

    fn handler_identifier(&self) -> String {
        format!(
            "{}::handler(req)",
            self.clean_path().split("/").collect::<Vec<_>>().join("::")
        )
    }

    fn handle_fn_match_arm(&self) -> String {
        format!(
            "\"{}\" => {},",
            self.route_path(),
            self.handler_identifier()
        )
    }

    fn mod_name(&self) -> String {
        let clean_path = self.clean_path();
        let split_path: Vec<_> = clean_path.split("/").collect();
        String::from(split_path[split_path.len() - 1])
    }

    fn has_handler(&self) -> bool {
        if !metadata(&self.path).unwrap().is_file() {
            println!("Path {} is not a directory", self.path);
            exit(-1);
        }

        let content = read_to_string(&self.path).unwrap();

        content.contains("pub fn handler(")
    }

    fn get_mod(&self) -> String {
        let padding = vec!["    "; self.nesting as usize].join("");
        let mut mod_str = format!("{}pub mod {}", padding, self.mod_name());

        if let Some(children) = &self.children {
            let sub_mods = &children
                .iter()
                .map(|c| c.get_mod())
                .collect::<Vec<_>>()
                .join("\n");
            mod_str += &format!(" {{\n{}\n{}}}", sub_mods, padding);
        } else {
            mod_str += ";";
        }

        mod_str
    }

    fn get_handle_fn_arms(&self) -> Vec<String> {
        let mut arms: Vec<String> = vec![];

        if let Some(children) = &self.children {
            arms.append(
                &mut children
                    .iter()
                    .flat_map(|c| c.get_handle_fn_arms())
                    .collect::<Vec<_>>(),
            );
        } else if self.has_handler() {
            arms.push(self.handle_fn_match_arm());
        }

        arms
    }
}

static DEFINES: &str = "
$modules

$ext_defines

fn handle(req: Request) -> Response {
    let clean_path = req.path.split(\"?\")
        .collect::<Vec<_>>()[0].trim_matches('/');

    match clean_path {
        $handles
        _ => Response {
            code: 404,
            body: Some(String::from(\"Not found\"))
        },
    }
}
";

fn get_defines(base_route: Route) -> String {
    let modules = base_route.get_mod();
    let handles = base_route.get_handle_fn_arms().join("\n");

    DEFINES
        .replace("$modules", &modules)
        .replace("$handles", &handles)
        .replace("$ext_defines", ext_defines)
}

#[proc_macro]
pub fn make_server(routes_path: TokenStream) -> TokenStream {
    let path = routes_path.to_string().replace("\"", "");
    let base_route = Route::base(path);

    let defines = get_defines(base_route);

    defines.parse().unwrap()
}
