use yew::prelude::*;
use yew_router::prelude::*;

mod pages;
mod services;
mod models;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/login")]
    Login,
    #[at("/signup")]
    Signup,
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <h1>{ "Welcome to Key Manager!" }</h1> },
        Route::Login => html! { <pages::login::Login /> },
        Route::Signup => html! { <pages::signup::Signup /> },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <nav class="bg-light py-2">
                <div class="container d-flex align-items-center justify-content-between bg-light">
                    <h3 class="fs-5 m-0">{ "🔐 Key Manager" }</h3>
                    <div class="d-flex gap-2">
                        <button type="button" class="btn btn-dark">
                            <Link<Route> to={Route::Login} classes="nav-link">{ "Login" }</Link<Route>>
                        </button>
                        <button type="button" class="btn btn-dark">
                            <Link<Route> to={Route::Signup} classes="nav-link">{ "Signup" }</Link<Route>>
                        </button>
                    </div>
                </div>
            </nav>
            <main class="container my-3">
                <Switch<Route> render={switch} />
            </main>
        </BrowserRouter>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
