mod defines;
mod mime_type;
mod route;

use {defines::DEFINES, proc_macro::TokenStream, route::Route};

fn get_defines(base_route: Route) -> String {
    let modules = base_route.get_mod(None);
    let handlers = base_route.get_handlers().join("\n");

    DEFINES
        .replace("$modules", &modules)
        .replace("$handlers", &handlers)
}

#[proc_macro]
pub fn make_server(_: TokenStream) -> TokenStream {
    let base_route = Route::base();
    get_defines(base_route).parse().unwrap()
}
