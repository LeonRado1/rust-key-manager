use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use regex::Regex;
use log::info;

use crate::services::auth;
use crate::models::user;

#[function_component(Signup)]
pub fn signup() -> Html {
    let email_address = use_state(|| String::new());
    let username = use_state(|| String::new());
    let password = use_state(|| String::new());
    let confirm_password = use_state(|| String::new());

    let error_message = use_state(|| String::new());

    let oninput_email_address = {
        let email_address = email_address.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            email_address.set(input.value());
        })
    };

    let oninput_username = {
        let username = username.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            username.set(input.value());
        })
    };

    let oninput_password = {
        let password = password.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            password.set(input.value());
        })
    };

    let oninput_confirm_password = {
        let confirm_password = confirm_password.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            confirm_password.set(input.value());
        })
    };

    let is_invalid = (*email_address).is_empty()
        || (*username).is_empty()
        || (*password).is_empty()
        || (*confirm_password).is_empty();

    pub fn is_password_valid(password: &str, confirm_password: &str) -> bool {
        password == confirm_password
    }
    
    pub fn is_email_valid(email: &str) -> bool {
        match Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$") {
            Ok(email_regex) => email_regex.is_match(email),
            Err(_) => false
        }
    }

    let on_signup = {
        let email_address = email_address.clone();
        let username = username.clone();
        let password = password.clone();
        let confirm_password = confirm_password.clone();

        let error_message = error_message.clone();
        
        Callback::from(move |_e: MouseEvent| {
            if !(*error_message).is_empty() {
                error_message.set(String::new());
            }

            if !is_email_valid(&email_address) {
                error_message.set("Invalid email format.".to_string());
                return;
            }

            if !is_password_valid(&password, &confirm_password) {
                error_message.set("Passwords do not match.".to_string());
                return;
            }

            let new_user = user::SignupRequest {
                username: (*username).clone(),
                email: (*email_address).clone(),
                password: (*password).clone()
            };

            let error_message = error_message.clone();

            spawn_local(async move {
                match auth::signup(new_user).await {
                    Ok(response) => {
                        info!("{:?}", response);
                    }
                    Err(err) => {
                        error_message.set(format!("Signup failed: {}", err));
                    }
                }
            });
        })
    };

    html! {
        <div class="col-lg-4 mx-auto">
            <h2 class="text-center mb-3">{ "Signup" }</h2>
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
                    <label class="form-label" for="username">{ "Username" }</label>
                    <input
                        id="username"
                        type="text"
                        class="form-control"
                        value={(*username).clone()}
                        oninput={oninput_username}
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
                <div class="mb-3">
                    <label class="form-label" for="confirm-password">{ "Confirm Password" }</label>
                    <input
                        id="confirm-password"
                        type="password"
                        class="form-control"
                        value={(*confirm_password).clone()}
                        oninput={oninput_confirm_password}
                    />
                </div>
                <div class="text-center">
                    <button
                        class="btn btn-success mx-auto"
                        type="button"
                        disabled={is_invalid}
                        onclick={on_signup}
                    >
                        <i class="bi bi-person-add me-1"></i>
                        { "Signup" }
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
