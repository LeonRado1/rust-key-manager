use yew::prelude::*;
use yew_router::hooks::use_navigator;
use yew_router::prelude::Link;
use crate::components::app_router::Route;
use crate::constants::key_types::{get_btn_type_class, API_KEY, PASSWORD, SSH_KEY, TOKEN};
use crate::context::user_context::use_user_context;

#[function_component(AddMenu)]
pub fn add_menu() -> Html {

    let user_ctx = use_user_context();
    let navigator = use_navigator().unwrap();

    {
        let user_ctx = user_ctx.clone();
        let navigator = navigator.clone();

        use_effect_with(user_ctx.clone(), move |ctx| {
            if ctx.user.is_none() {
                navigator.push(&Route::Login);
            }
            || ()
        });
    }

    html! {
        <div class="col-lg-4 mx-auto">
            <h2 class="text-center mb-5">{ "Select Key Type" }</h2>
            <div class="d-flex flex-column gap-4">
                <Link<Route>
                    to={Route::AddKey { id: PASSWORD }}
                    classes={classes!("btn", get_btn_type_class(PASSWORD))}
                >
                    <i class="bi bi-key-fill me-2"></i>
                    {"Password"}
                </Link<Route>>

                <Link<Route>
                    to={Route::AddKey { id: TOKEN }}
                    classes={classes!("btn", get_btn_type_class(TOKEN))}
                >
                    <i class="bi bi-lock-fill me-2"></i>
                    {"Token"}
                </Link<Route>>

                <Link<Route>
                    to={Route::AddKey { id: API_KEY }}
                    classes={classes!("btn", get_btn_type_class(API_KEY))}
                >
                    <i class="bi bi-database-fill me-2"></i>
                    {"API Key"}
                </Link<Route>>

                <Link<Route>
                    to={Route::AddKey { id: SSH_KEY }}
                    classes={classes!("btn", get_btn_type_class(SSH_KEY))}
                >
                    <i class="bi bi-hdd-stack-fill me-2"></i>
                    {"SSH Key"}
                </Link<Route>>
            </div>
        </div>
    }
}
