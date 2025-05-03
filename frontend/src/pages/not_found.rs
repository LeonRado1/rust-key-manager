use yew::prelude::*;
use yew_router::prelude::Link;
use crate::components::app_router::Route;

#[function_component(NotFound)]
pub fn not_found() -> Html {
    html! {
        <div class="container text-center mt-5">
            <h1 class="text-danger">{"404!"}</h1>
            <p>{"Not found. The page you're looking for doesn't exist."}</p>
            <Link<Route> to={Route::Dashboard} classes="btn btn-danger">
                <i class="bi bi-house-fill me-1"></i>
                {"Go Home"}
            </Link<Route>>
        </div>
    }
}
