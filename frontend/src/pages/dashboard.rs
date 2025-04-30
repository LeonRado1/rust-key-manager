use yew::prelude::*;
use yew_router::hooks::use_navigator;
use crate::components::app_router::Route;
use crate::context::user_context::use_user_context;

#[function_component(Dashboard)]
pub fn dashboard() -> Html {

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
        <h1>
           {"Welcome to the dashboard!"}
        </h1>
    }
}
