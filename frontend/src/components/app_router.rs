use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages;
use crate::context::user_context::use_user_context;
use crate::models::user::User;
use crate::helpers::storage;

#[derive(Clone, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Dashboard,
    #[at("/login")]
    Login,
    #[at("/register")]
    Register,
    #[not_found]
    #[at("/404")]
    NotFound,
}

fn switch(route: Route) -> Html {
    match route {
        Route::Dashboard => html! { <pages::dashboard::Dashboard /> },
        Route::Login => html! { <pages::login::Login /> },
        Route::Register => html! { <pages::register::Register /> },
        Route::NotFound => html! { <pages::not_found::NotFound /> },
    }
}

#[function_component(AppRouter)]
pub fn app_router() -> Html {

    let user_ctx = use_user_context();
    let navigator = use_navigator().unwrap();

    let on_logout = {
        let user_ctx = user_ctx.clone();
        let navigator = navigator.clone();

        Callback::from(move |_| {
            let _ = storage::remove_token();

            user_ctx.set_user.emit(None::<User>);
            navigator.push(&Route::Login);
        })
    };

    html! {
        <>
            <nav class="bg-light py-2">
                <div class="container d-flex align-items-center justify-content-between bg-light">
                    <h3 class="fs-5 m-0">{ "🔐 Key Manager" }</h3>
                    <div class="d-flex aling-items-center gap-2">
                        {
                            if let Some(user) = &user_ctx.user {
                                html! {
                                    <>
                                        <div class="d-flex align-items-center gap-1 me-2">
                                            <i class="bi bi-person-fill"></i>
                                            <span>
                                                {"Welcome, "}
                                                <span class="fw-bold">{&user.username}</span>
                                            </span>
                                        </div>
                                        <Link<Route> to={Route::Dashboard} classes="btn btn-outline-dark">
                                            <i class="bi bi-bar-chart-fill me-1"></i>
                                            { "Dashboard" }
                                        </Link<Route>>
                                        <Link<Route> to={Route::Register} classes="btn btn-outline-dark">
                                            <i class="bi bi-file-earmark-plus-fill me-1"></i>
                                            { "Add Key" }
                                        </Link<Route>>
                                        <Link<Route> to={Route::Register} classes="btn btn-outline-dark">
                                            <i class="bi bi-gear-fill me-1"></i>
                                            {"Settings"}
                                        </Link<Route>>
                                        <button
                                            type="button"
                                            class="btn btn-outline-danger"
                                            onclick={on_logout}
                                        >
                                            <i class="bi bi-box-arrow-right me-1"></i>
                                            { "Logout" }
                                        </button>
                                    </>
                                }
                            } else {
                                html! {
                                    <>
                                        <Link<Route> to={Route::Login} classes="btn btn-dark">
                                            <i class="bi bi-box-arrow-in-right me-1"></i>
                                            { "Login" }
                                        </Link<Route>>
                                        <Link<Route> to={Route::Register} classes="btn btn-dark">
                                            <i class="bi bi-person-plus me-1"></i>
                                            { "Register" }
                                        </Link<Route>>
                                    </>
                                }
                            }
                        }
                    </div>
                </div>
            </nav>
            <main class="container my-3">
                <Switch<Route> render={switch} />
            </main>
        </>
    }
}
