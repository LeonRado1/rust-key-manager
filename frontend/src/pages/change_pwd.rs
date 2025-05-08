use regex::Regex;
use web_sys::HtmlInputElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use yew_router::prelude::Link;
use crate::components::app_router::Route;
use crate::context::user_context::use_user_context;
use crate::helpers::storage;
use crate::helpers::string_utils::string_or_none;
use crate::models::user::{ChangePasswordRequest, ChangeUserRequest};
use crate::services::auth;

#[function_component(ChangePwd)]
pub fn change_pwd() -> Html {
    let change_request = use_state(ChangePasswordRequest::default);

    let error_message = use_state(|| String::new());
    let success_message = use_state(|| String::new());

    let user_ctx = use_user_context();
    let navigator = use_navigator().unwrap();

    {
        let user_ctx = user_ctx.clone();
        let navigator = navigator.clone();

        use_effect_with(user_ctx.clone(), move |ctx| {
            if !ctx.is_loading && ctx.user.is_some() {
                navigator.push(&Route::Dashboard);
            }
            || ()
        });
    }

    let on_email_change = {
        let change_request = change_request.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut details = (*change_request).clone();
            details.email = input.value();
            change_request.set(details);
        })
    };

    let on_code_change = {
        let change_request = change_request.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut details = (*change_request).clone();
            details.recovery_code = input.value();
            change_request.set(details);
        })
    };

    let on_password_change = {
        let change_request = change_request.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut details = (*change_request).clone();
            details.new_password = input.value();
            change_request.set(details);
        })
    };
    
    let is_change_invalid = (*change_request).email.is_empty() 
        || (*change_request).recovery_code.is_empty()
        || (*change_request).new_password.is_empty();

    pub fn is_email_valid(email: &str) -> bool {
        match Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$") {
            Ok(email_regex) => email_regex.is_match(email),
            Err(_) => false
        }
    }

    let on_change = {
        let change_request = change_request.clone();

        let success_message = success_message.clone();
        let error_message = error_message.clone();

        Callback::from(move |_e: MouseEvent| {
            let change_request = change_request.clone();

            let success_message = success_message.clone();
            let error_message = error_message.clone();

            if !(*error_message).is_empty() {
                error_message.set(String::new());
            }

            if !(*success_message).is_empty() {
                success_message.set(String::new());
            }
            
            if !is_email_valid(&(*change_request).email.clone()) {
                error_message.set("Invalid email format.".to_string());
                return;
            }

            spawn_local(async move {
                match auth::change_password((*change_request).clone()).await {
                    Ok(_) => {
                        success_message.set("Your password was changed successfully.".to_string());
                        change_request.set(ChangePasswordRequest::default());
                    }
                    Err(err) => error_message.set(err),
                }
            });
        })
    };

    html! {
         <div class="col-lg-4 mx-auto">
            <h2 class="text-center mb-3">{ "Change Password" }</h2>
            if !success_message.is_empty() {
                <div class="alert alert-success mb-3 d-flex align-items-center justify-content-between">
                    <div>
                        {(*success_message).clone()}
                    </div>
                    <Link<Route>
                        to={Route::Login}
                        classes="btn btn-success btn-sm"
                    >
                        {"Login"}
                    </Link<Route>>
                </div>
            }
            <form>
                <div class="mb-3">
                    <label class="form-label" for="email">{ "Email" }</label>
                    <input
                        id="email"
                        type="email"
                        class="form-control"
                        value={(*change_request).email.clone()}
                        oninput={on_email_change}
                    />
                </div>
                <div class="mb-4">
                    <label class="form-label" for="password">{ "New Password" }</label>
                    <input
                        id="password"
                        type="password"
                        class="form-control"
                        value={(*change_request).new_password.clone()}
                        oninput={on_password_change}
                    />
                </div>
                <div class="pt-3 border-top">
                    <label class="form-label" for="code">{ "Recovery Code" }</label>
                    <input
                        id="code"
                        type="text"
                        class="form-control"
                        value={(*change_request).recovery_code.clone()}
                        oninput={on_code_change}
                    />
                </div>
                <div class="text-center mt-4">
                    <button
                        class="btn btn-outline-primary mx-auto w-50"
                        type="button"
                        disabled={is_change_invalid}
                        onclick={on_change}
                    >
                        <i class="bi bi-arrow-up-right-square-fill me-2"></i>
                        { "Change" }
                    </button>
                </div>
                if !error_message.is_empty() {
                    <div class="alert alert-danger mt-3">
                        {(*error_message).clone()}
                    </div>
                }
            </form>
        </div>
    }
}
