use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use regex::Regex;
use log::info;

use crate::services::auth;
use crate::models::user;

#[function_component(Login)]
pub fn login() -> Html {
    let email_address = use_state(|| String::new());
    let password = use_state(|| String::new());

    let error_message = use_state(|| String::new());

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
        
        Callback::from(move |_e: MouseEvent| {
            if !(*error_message).is_empty() {
                error_message.set(String::new());
            }

            if !is_email_valid(&email_address) {
                error_message.set("Invalid email format.".to_string());
                return;
            }

            let user = user::LoginRequest {
                email: (*email_address).clone(),
                password: (*password).clone()
            };

            let error_message = error_message.clone();
            
            spawn_local(async move {
                match auth::login(user).await {
                    Ok(response) => {
                        info!("{:?}", response);
                    }
                    Err(err) => {
                        error_message.set(format!("Login failed: {}", err));
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
                        class="btn btn-success mx-auto"
                        type="button"
                        disabled={is_invalid}
                        onclick={on_login}
                    >
                        <i class="bi bi-person-check me-1"></i>
                        { "Login" }
                    </button>
                </div>
                if !error_message.is_empty() {
                    <div class="bg-danger mt-3 p-2">
                        <p class="mb-1 fw-bold">
                            <i class="bi bi-exclamation-square-fill me-1"></i>
                            {"Error"}
                        </p>
                        <p class="m-0">
                            {(*error_message).clone()}
                        </p>
                    </div>
                }
            </form>
        </div>
    }
}
