mod defines;
mod route;

use {crate::route::Route, defines::DEFINES as ext_defines, proc_macro::TokenStream};

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
pub fn make_server(ts: TokenStream) -> TokenStream {
    let args = ts.into_iter().collect::<Vec<_>>();

    let routes_path = format!("{}", &args[0]).replace("\"", "");
    let base_route = Route::base(routes_path);

    get_defines(base_route).parse().unwrap()
}
