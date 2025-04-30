use yew::prelude::*;
use yew_router::prelude::*;
use crate::components::app_router::AppRouter;
use crate::context::user_context::UserContextProvider;

mod pages;
mod services;
mod models;
mod context;
mod helpers;
mod components;

#[function_component(App)]
fn app() -> Html {
    html! {
        <UserContextProvider>
            <BrowserRouter>
                <AppRouter></AppRouter>
            </BrowserRouter>
        </UserContextProvider>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
