use regex::Regex;
use web_sys::HtmlInputElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use crate::components::app_router::Route;
use crate::context::user_context::use_user_context;
use crate::helpers::storage;
use crate::helpers::string_utils::string_or_none;
use crate::models::user::{ChangeUserRequest, User};
use crate::services::auth;

#[function_component(Settings)]
pub fn settings() -> Html {
    let change_request = use_state(ChangeUserRequest::default);

    let error_message = use_state(|| String::new());
    let success_message = use_state(|| String::new());
    let show_delete_modal = use_state(|| false);

    let user_ctx = use_user_context();
    let navigator = use_navigator().unwrap();

    {
        let user_ctx = user_ctx.clone();
        let navigator = navigator.clone();

        use_effect_with(user_ctx.clone(), move |ctx| {
            if !ctx.is_loading && ctx.user.is_none() {
                navigator.push(&Route::Login);
            }
            || ()
        });
    }

    let on_username_change = {
        let change_request = change_request.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut details = (*change_request).clone();
            details.username = string_or_none(input.value());
            change_request.set(details);
        })
    };

    let on_email_change = {
        let change_request = change_request.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut details = (*change_request).clone();
            details.email = string_or_none(input.value());
            change_request.set(details);
        })
    };

    let on_password_change = {
        let change_request = change_request.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut details = (*change_request).clone();
            details.password = input.value();
            change_request.set(details);
        })
    };

    let is_change_invalid =
        ((*change_request).username.is_none() && (*change_request).email.is_none())
        || (*change_request).password.is_empty();

    let is_delete_invalid =  (*change_request).password.is_empty();

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

        let user_ctx = user_ctx.clone();

        Callback::from(move |_e: MouseEvent| {
            let change_request = (*change_request).clone();

            let success_message = success_message.clone();
            let error_message = error_message.clone();

            let user_ctx = user_ctx.clone();

            if !(*error_message).is_empty() {
                error_message.set(String::new());
            }

            if !(*success_message).is_empty() {
                success_message.set(String::new());
            }

            if let Some(email) = &change_request.email {
                if !is_email_valid(email) {
                    error_message.set("Invalid email format.".to_string());
                    return;
                }
            }

            spawn_local(async move {
                if let Ok(token) = storage::get_token() {
                    match auth::change_user_details(&token, change_request).await {
                        Ok(response) => {
                            user_ctx.set_user.emit(Some(response));
                            success_message.set("Account details were updated successfully.".to_string());
                        }
                        Err(err) => error_message.set(err),
                    }
                }
            });
        })
    };

    let on_delete = {
        let change_request = change_request.clone();

        let success_message = success_message.clone();
        let error_message = error_message.clone();
        let show_delete_modal = show_delete_modal.clone();

        let user_ctx = user_ctx.clone();
        let navigator = navigator.clone();

        Callback::from(move |_e: MouseEvent| {
            let change_request = (*change_request).clone();

            let success_message = success_message.clone();
            let error_message = error_message.clone();
            let show_delete_modal = show_delete_modal.clone();

            let user_ctx = user_ctx.clone();
            let navigator = navigator.clone();

            if !(*error_message).is_empty() {
                error_message.set(String::new());
            }

            if !(*success_message).is_empty() {
                success_message.set(String::new());
            }

            spawn_local(async move {
                if let Ok(token) = storage::get_token() {
                    match auth::delete_user(&token, change_request).await {
                        Ok(_) => {
                            let _ = storage::remove_token();

                            user_ctx.set_user.emit(None::<User>);
                            navigator.push(&Route::Login);
                        }
                        Err(err) => {
                            error_message.set(err);
                            show_delete_modal.set(false);
                        }
                    }
                }
            });
        })
    };

    let on_open_delete_modal = {
        let show_delete_modal = show_delete_modal.clone();
        Callback::from(move |_e: MouseEvent| {
            show_delete_modal.set(true);
        })
    };
    let on_close_delete_modal = {
        let show_delete_modal = show_delete_modal.clone();
        Callback::from(move |_e: MouseEvent| {
            show_delete_modal.set(false);
        })
    };

    html! {
         <div class="col-lg-4 mx-auto">
            <h2 class="text-center mb-3">{ "Settings" }</h2>
            if !success_message.is_empty() {
                <div class="alert alert-success alert-dismissible fade show mb-3">
                    {(*success_message).clone()}
                    <button
                        type="button"
                        class="btn-close"
                        onclick={
                            let success_message = success_message.clone();
                            Callback::from(move |_| success_message.set(String::new()))
                        }
                    />
                </div>
            }
            <form>
                <div class="mb-3">
                    <label class="form-label" for="username">{ "Username" }</label>
                    <input
                        id="username"
                        type="text"
                        class="form-control"
                        value={(*change_request).username.clone().unwrap_or_default()}
                        oninput={on_username_change}
                    />
                </div>
                <div class="mb-4">
                    <label class="form-label" for="email">{ "Email" }</label>
                    <input
                        id="email"
                        type="email"
                        class="form-control"
                        value={(*change_request).email.clone().unwrap_or_default()}
                        oninput={on_email_change}
                    />
                </div>
                <div class="pt-3 border-top">
                    <label class="form-label" for="password">{ "Password" }</label>
                    <input
                        id="password"
                        type="password"
                        class="form-control"
                        value={(*change_request).password.clone()}
                        oninput={on_password_change}
                    />
                </div>
                <div class="text-center mt-4">
                    <button
                        class="btn btn-outline-success mx-auto w-50"
                        type="button"
                        disabled={is_change_invalid}
                        onclick={on_change}
                    >
                        <i class="bi bi-floppy2-fill me-2"></i>
                        { "Save Details" }
                    </button>
                </div>
                <div class="text-center mt-3">
                    <button
                        class="btn btn-outline-danger mx-auto w-50"
                        type="button"
                        disabled={is_delete_invalid}
                        onclick={on_open_delete_modal}
                    >
                        <i class="bi bi-archive-fill me-2"></i>
                        { "Delete Account" }
                    </button>
                </div>
                if !error_message.is_empty() {
                    <div class="alert alert-danger mt-3">
                        {(*error_message).clone()}
                    </div>
                }
            </form>

            // Delete Account Modal
            if *show_delete_modal {
                <div class="modal fade show" style="display: block;" tabindex="-1" role="dialog">
                    <div class="modal-dialog modal-dialog-centered">
                        <div class="modal-content">
                            <div class="modal-header border-0">
                                <h5 class="modal-title fw-bold">{ "Confirm Account Deletion" }</h5>
                                <button
                                    type="button"
                                    class="btn-close"
                                    onclick={on_close_delete_modal.clone()}
                                ></button>
                            </div>
                            <div class="modal-body">
                                <p class="text-muted">
                                    { "Are you sure you want to delete your account? This action cannot be undone." }
                                </p>
                            </div>
                            <div class="modal-footer border-0">
                                <button
                                    type="button"
                                    class="btn btn-secondary"
                                    onclick={on_close_delete_modal}
                                >
                                    { "Cancel" }
                                </button>
                                <button
                                    type="button"
                                    class="btn btn-danger"
                                    onclick={on_delete}
                                >
                                    <i class="bi bi-trash-fill me-2"></i>
                                    { "Delete Account" }
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="modal-backdrop fade show"></div>
            }
        </div>
    }
}
