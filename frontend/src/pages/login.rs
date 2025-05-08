use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use regex::Regex;
use yew_router::hooks::use_navigator;

use crate::components::app_router::Route;
use crate::context::user_context::use_user_context;
use crate::helpers::storage;
use crate::services::auth;
use crate::models::user::LoginRequest;

#[function_component(Login)]
pub fn login() -> Html {
    let email_address = use_state(|| String::new());
    let password = use_state(|| String::new());

    let error_message = use_state(|| String::new());

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

    let oninput_email_address = {
        let email_address = email_address.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            email_address.set(input.value());
        })
    };

    let oninput_password = {
        let password = password.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            password.set(input.value());
        })
    };

    let is_invalid = (*email_address).is_empty()
        || (*password).is_empty();
    
    pub fn is_email_valid(email: &str) -> bool {
        match Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$") {
            Ok(email_regex) => email_regex.is_match(email),
            Err(_) => false
        }
    }

    let on_login = {
        let email_address = email_address.clone();
        let password = password.clone();

        let error_message = error_message.clone();

        let user_ctx = user_ctx.clone();
        let navigator = navigator.clone();
        
        Callback::from(move |_e: MouseEvent| {
            if !(*error_message).is_empty() {
                error_message.set(String::new());
            }

            if !is_email_valid(&email_address) {
                error_message.set("Invalid email format.".to_string());
                return;
            }

            let user = LoginRequest {
                email: (*email_address).clone(),
                password: (*password).clone()
            };

            let error_message = error_message.clone();

            let user_ctx = user_ctx.clone();
            let navigator = navigator.clone();
            
            spawn_local(async move {
                match auth::login(user).await {
                    Ok(response) => {
                        if let Ok(_) = storage::save_token(&response.token) {
                            user_ctx.set_user.emit(Some(response.user));
                            navigator.push(&Route::Dashboard);
                        } else {
                            error_message.set("Failed to save authentication token.".to_string());
                        }
                    }
                    Err(err) => {
                        error_message.set(err);
                    }
                }
            });
        })
    };

    html! {
        <div class="col-lg-4 mx-auto">
            <h2 class="text-center mb-3">{ "Login" }</h2>
            <form>
                <div class="mb-3">
                    <label class="form-label" for="email">{ "Email" }</label>
                    <input
                        id="email"
                        type="email"
                        class="form-control"
                        required={true}
                        value={(*email_address).clone()}
                        oninput={oninput_email_address}
                    />
                </div>
                <div class="mb-3">
                    <label class="form-label" for="password">{ "Password" }</label>
                    <input
                        id="password"
                        type="password"
                        class="form-control"
                        value={(*password).clone()}
                        oninput={oninput_password}
                    />
                </div>
                <div class="text-center">
                    <button
                        class="btn btn-outline-success mx-auto"
                        type="button"
                        disabled={is_invalid}
                        onclick={on_login}
                    >
                        <i class="bi bi-box-arrow-in-right me-1"></i>
                        { "Login" }
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
