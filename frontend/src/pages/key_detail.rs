use web_sys::{window, File, HtmlInputElement};
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use crate::components::app_router::Route;
use crate::constants::key_types::{PASSWORD, SSH_KEY, get_type_name, get_type_class, TOKEN};
use crate::context::user_context::use_user_context;
use crate::helpers::date::{format_date, DateStatus, validate_date};
use crate::helpers::storage;
use crate::helpers::string_utils::{copy_to_clipboard, generate_password, string_or_none};
use crate::models::key::{Key, KeyRequest, UpdateKeyRequest};
use crate::services::keys;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: i32,
}

#[function_component(KeyDetail)]
pub fn key_detail(props: &Props) -> Html {

    let key = use_state(|| None::<Key>);
    let update_request = use_state(UpdateKeyRequest::default);

    let error_message = use_state(|| String::new());
    let success_message = use_state(|| String::new());

    let show_key_value = use_state(|| false);
    let show_new_password = use_state(|| false);

    let user_ctx = use_user_context();
    let navigator = use_navigator().unwrap();

    {
        let user_ctx = user_ctx.clone();
        let navigator = navigator.clone();

        let key = key.clone();

        let props_id = props.id;

        use_effect_with(user_ctx.clone(), move |ctx| {
            if ctx.user.is_none() {
                navigator.push(&Route::Login);
            }
            else {
                spawn_local(async move {
                    if let Ok(token) = storage::get_token() {
                        match keys::get_key_detail(&token, props_id).await {
                            Ok(response) => {
                                key.set(Some(response));
                            }
                            Err(_e) => ()
                        }
                    }
                });
            }
            || ()
        });
    }

    let on_clipboard = {
        let key = key.clone();

        Callback::from(move |_| {
            if let Some(key_data) = (*key).as_ref() {
                let text = if key_data.key_type_id == SSH_KEY {
                    key_data.key_pair_value.clone().unwrap_or_default()
                } else {
                    key_data.key_value.clone()
                };

                copy_to_clipboard(&text);
            }
        })
    };

    let toggle_key_value_visibility = {
        let show_key_value = show_key_value.clone();
        Callback::from(move |_| {
            show_key_value.set(!*show_key_value);
        })
    };

    let toggle_new_password_visibility = {
        let show_new_password = show_new_password.clone();
        Callback::from(move |_| {
            show_new_password.set(!*show_new_password);
        })
    };

    let generate_value = {
        let update_request = update_request.clone();
        Callback::from(move |_e: MouseEvent| {
            let generated = generate_password(16);
            let mut new_request = (*update_request).clone();
            new_request.new_password = Some(generated.clone());
            update_request.set(new_request);
        })
    };

    let on_password_change = {
        let update_request = update_request.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_request = (*update_request).clone();
            new_request.new_password = string_or_none(input.value());
            update_request.set(new_request);
        })
    };

    let on_expiration_change = {
        let update_request = update_request.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_request = (*update_request).clone();
            new_request.new_expiration_date = string_or_none(input.value());
            update_request.set(new_request);
        })
    };

    let on_export_ssh = {
        let key = key.clone();
        let error_message = error_message.clone();

        Callback::from(move |_e: MouseEvent| {
            let key = key.clone();
            let error_message = error_message.clone();

            if !(*error_message).is_empty() {
                error_message.set(String::new());
            }

            spawn_local(async move {
                match keys::export_key(&(*key).clone().unwrap().key_value).await {
                    Ok(_) => (),
                    Err(err) => {
                        error_message.set(err);
                    }
                }
            });
        })
    };

    let on_extend = {
        let key = key.clone();
        let update_request = update_request.clone();
        
        let success_message = success_message.clone();
        let error_message = error_message.clone();

        Callback::from(move |_e: MouseEvent| {
            let key = key.clone();
            let update_request = update_request.clone();
            
            let success_message = success_message.clone();
            let error_message = error_message.clone();

            if !(*error_message).is_empty() {
                error_message.set(String::new());
            }

            if !(*success_message).is_empty() {
                success_message.set(String::new());
            }

            spawn_local(async move {
                if let Ok(token) = storage::get_token() {
                    let mut req = UpdateKeyRequest::default();
                    req.key_id = (*key).clone().unwrap().id;
                    req.new_expiration_date = (*update_request).clone().new_expiration_date;

                    match keys::extend_key(&token, req).await {
                        Ok(updated_key) => {
                            key.set(Some(updated_key));
                            success_message.set("Your request was processed successfully.".to_string());
                            update_request.set(UpdateKeyRequest::default());
                        }
                        Err(err) => error_message.set(err),
                    }
                }
            });
        })
    };

    let on_extend_rotate = {
        let key = key.clone();
        let update_request = update_request.clone();

        let success_message = success_message.clone();
        let error_message = error_message.clone();

        Callback::from(move |_e: MouseEvent| {
            let key = key.clone();
            let update_request = update_request.clone();

            let success_message = success_message.clone();
            let error_message = error_message.clone();

            if !(*error_message).is_empty() {
                error_message.set(String::new());
            }

            if !(*success_message).is_empty() {
                success_message.set(String::new());
            }

            spawn_local(async move {
                if let Ok(token) = storage::get_token() {
                    let mut req = UpdateKeyRequest::default();
                    req.key_id = (*key).clone().unwrap().id;
                    req.new_expiration_date = (*update_request).clone().new_expiration_date;

                    match keys::extend_rotate_key(&token, req).await {
                        Ok(updated_key) => {
                            key.set(Some(updated_key));
                            success_message.set("Your request was processed successfully.".to_string());
                            update_request.set(UpdateKeyRequest::default());
                        }
                        Err(err) => error_message.set(err),
                    }
                }
            });
        })
    };

    let on_change_password = {
        let key = key.clone();
        let update_request = update_request.clone();

        let success_message = success_message.clone();
        let error_message = error_message.clone();

        Callback::from(move |_e: MouseEvent| {
            let key = key.clone();
            let update_request = update_request.clone();

            let success_message = success_message.clone();
            let error_message = error_message.clone();

            if !(*error_message).is_empty() {
                error_message.set(String::new());
            }

            if !(*success_message).is_empty() {
                success_message.set(String::new());
            }

            spawn_local(async move {
                if let Ok(token) = storage::get_token() {
                    let mut req = UpdateKeyRequest::default();
                    req.key_id = (*key).clone().unwrap().id;
                    req.new_password = (*update_request).clone().new_password;

                    match keys::change_key(&token, req).await {
                        Ok(updated_key) => {
                            key.set(Some(updated_key));
                            success_message.set("Your request was processed successfully.".to_string());
                            update_request.set(UpdateKeyRequest::default());
                        }
                        Err(err) => error_message.set(err),
                    }
                }
            });
        })
    };

    let on_rotate = {
        let key = key.clone();
        
        let success_message = success_message.clone();
        let error_message = error_message.clone();

        Callback::from(move |_e: MouseEvent| {
            let key = key.clone();

            let success_message = success_message.clone();
            let error_message = error_message.clone();

            if !(*error_message).is_empty() {
                error_message.set(String::new());
            }

            if !(*success_message).is_empty() {
                success_message.set(String::new());
            }
            
            let key_id = (*key).clone().unwrap().id;

            spawn_local(async move {
                if let Ok(token) = storage::get_token() {
                    match keys::rotate_key(&token, key_id).await {
                        Ok(updated_key) => {
                            key.set(Some(updated_key));
                            success_message.set("Your request was processed successfully.".to_string());
                        }
                        Err(err) => error_message.set(err),
                    }
                }
            });
        })
    };

    let on_revoke = {
        let key = key.clone();

        let success_message = success_message.clone();
        let error_message = error_message.clone();
        
        let navigator = navigator.clone();

        Callback::from(move |_e: MouseEvent| {
            let key = key.clone();

            let success_message = success_message.clone();
            let error_message = error_message.clone();
            
            let navigator = navigator.clone();

            if !(*error_message).is_empty() {
                error_message.set(String::new());
            }

            if !(*success_message).is_empty() {
                success_message.set(String::new());
            }

            let key_id = (*key).clone().unwrap().id;

            spawn_local(async move {
                if let Ok(token) = storage::get_token() {
                    match keys::revoke_key(&token, key_id).await {
                        Ok(_) => {
                            navigator.push(&Route::Dashboard);
                        }
                        Err(err) => error_message.set(err),
                    }
                }
            });
        })
    };

    let on_delete = {
        let key = key.clone();

        let success_message = success_message.clone();
        let error_message = error_message.clone();

        let navigator = navigator.clone();

        Callback::from(move |_e: MouseEvent| {
            let key = key.clone();

            let success_message = success_message.clone();
            let error_message = error_message.clone();

            let navigator = navigator.clone();

            if !(*error_message).is_empty() {
                error_message.set(String::new());
            }

            if !(*success_message).is_empty() {
                success_message.set(String::new());
            }

            let key_id = (*key).clone().unwrap().id;

            spawn_local(async move {
                if let Ok(token) = storage::get_token() {
                    match keys::delete_key(&token, key_id).await {
                        Ok(_) => {
                            navigator.push(&Route::Dashboard);
                        }
                        Err(err) => error_message.set(err),
                    }
                }
            });
        })
    };

    html! {
        <div class="col-lg-8 mx-auto my-5">
            if !(*success_message).is_empty() {
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

            if let Some(key_data) = (*key).clone() {
                <div class="card shadow-none">
                    <div class="card-header bg-light d-flex align-items-center justify-content-between py-3">
                        <h5 class="m-0">{&key_data.key_name}</h5>
                        <span class={classes!("badge", format!("text-{}", get_type_class(key_data.key_type_id)))}>
                            {get_type_name(key_data.key_type_id)}
                        </span>
                    </div>
                    <div class="card-body">
                        if key_data.key_description.is_some() {
                            <div class="mb-3">
                                <label class="form-label text-muted">{"Description"}</label>
                                <p class="mb-0">{key_data.key_description.unwrap_or_default()}</p>
                            </div>
                        }
                        if key_data.key_tag.is_some() {
                            <div class="mb-3">
                                <label class="form-label text-muted">{"Tag"}</label>
                                <div>
                                    <span class="badge rounded-pill text-bg-dark">
                                        {key_data.key_tag.unwrap_or_default()}
                                    </span>
                                </div>
                            </div>
                        }
                        if key_data.expiration_date.is_some() {
                            <div class="mb-3">
                                <label class="form-label text-muted">{"Expiration Date"}</label>
                                <p class="mb-0">
                                    {format_date(key_data.expiration_date)}
                                </p>
                            </div>
                        }
                        <div class="mb-3">
                            <label class="form-label text-muted">
                                {if key_data.key_type_id != SSH_KEY {"Key Value"} else {"Public Key Value"}}
                            </label>
                            <div class="input-group mb-3">
                                <input
                                    type={if *show_key_value { "text" } else { "password" }}
                                    class="form-control"
                                    value={
                                        if key_data.key_type_id != SSH_KEY {key_data.key_value.clone()}
                                        else {key_data.key_pair_value.unwrap_or_default()}
                                    }
                                    readonly=true
                                />
                                <button
                                    onclick={toggle_key_value_visibility}
                                    class="btn btn-outline-danger"
                                >
                                    if *show_key_value {
                                        <i class="bi bi-eye-slash"></i>
                                    }
                                    else {
                                        <i class="bi bi-eye"></i>
                                    }
                                </button>
                                <button
                                    class="btn btn-outline-dark"
                                    onclick={on_clipboard}
                                >
                                    <i class="bi bi-clipboard"></i>
                                </button>
                            </div>
                        </div>
                        if key_data.key_type_id == SSH_KEY {
                            <div class="mb-3">
                                <label class="form-label text-muted">
                                    {"Private Key Value"}
                                </label>
                                <div class="input-group mb-3">
                                    <input
                                        type="password"
                                        class="form-control"
                                        value={key_data.key_value.clone()}
                                        readonly=true
                                    />
                                    <button
                                        class="btn btn-outline-primary"
                                        onclick={on_export_ssh}
                                    >
                                        <i class="bi bi-download"></i>
                                    </button>
                                </div>
                            </div>
                        }
                        <div class="mb-3 pt-3 border-top">
                            <label class="form-label text-muted">{key_data.key_type}</label>
                            <div class="mb-3">
                                if [DateStatus::Expired, DateStatus::AboutToExpire]
                                    .contains(&validate_date(key_data.expiration_date))
                                {
                                    <div class={
                                        format!("alert {}",
                                            if validate_date(key_data.expiration_date) == DateStatus::Expired {"alert-danger"}
                                            else {"alert-warning"}
                                        )
                                    }>
                                        <div class="d-flex align-items-center gap-2 mb-3">
                                            {
                                                if validate_date(key_data.expiration_date) == DateStatus::Expired {
                                                    "This key is expired."
                                                } else {
                                                    "This key is about to expire."
                                                }
                                            }
                                        </div>
                                        <div class="mb-3">
                                            <div class="input-group">
                                                <input
                                                    type="text"
                                                    class="form-control"
                                                    value={(*update_request).new_expiration_date.clone().unwrap_or_default()}
                                                    placeholder="YY/MM/DD hh:mm:ss"
                                                    oninput={on_expiration_change}
                                                />
                                                <span class="input-group-text">
                                                    <i class="bi bi-calendar2-event"></i>
                                                </span>
                                            </div>
                                        </div>
                                        <div class="d-flex gap-2">
                                            <button
                                                class="btn btn-primary"
                                                disabled={(*update_request).new_expiration_date.is_none()}
                                                onclick={on_extend}
                                            >
                                                <i class="bi bi-calendar-plus me-2"></i>
                                                {"Extend"}
                                            </button>
                                            <button
                                                class="btn btn-warning"
                                                disabled={(*update_request).new_expiration_date.is_none()}
                                                onclick={on_extend_rotate}
                                            >
                                                <i class="bi bi-arrow-repeat me-2"></i>
                                                {"Extend & Rotate"}
                                            </button>
                                            <button 
                                                class="btn btn-danger"
                                                onclick={on_revoke}
                                            >
                                                <i class="bi bi-x-circle me-2"></i>
                                                {"Revoke"}
                                            </button>
                                        </div>
                                    </div>
                                }
                                else if key_data.key_type_id == PASSWORD {
                                    <div class="input-group">
                                        <input
                                            type={if *show_new_password { "text" } else { "password" }}
                                            class="form-control"
                                            value={(*update_request).new_password.clone().unwrap_or_default()}
                                            oninput={on_password_change}
                                        />
                                        <button
                                            type="button"
                                            class="btn btn-outline-danger"
                                            onclick={toggle_new_password_visibility}
                                        >
                                            if *show_new_password {
                                                <i class="bi bi-eye-slash"></i>
                                            } else {
                                                <i class="bi bi-eye"></i>
                                            }
                                        </button>
                                        <button
                                            type="button"
                                            class="btn btn-outline-dark"
                                            onclick={generate_value}
                                        >
                                            {"Generate"}
                                        </button>
                                    </div>
                                    <div class="d-flex gap-2 mt-3">
                                        <button
                                            class="btn btn-success"
                                            disabled={(*update_request).new_password.is_none()}
                                            onclick={on_change_password}
                                        >
                                            <i class="bi bi-floppy2-fill me-2"></i>
                                            {"Change"}
                                        </button>
                                        <button 
                                            class="btn btn-danger"
                                            onclick={on_delete}
                                        >
                                            <i class="bi bi-trash-fill me-2"></i>
                                            {"Delete"}
                                        </button>
                                    </div>
                                }
                                else {
                                    <div class="d-flex gap-2">
                                        <button 
                                            class="btn btn-warning"
                                            onclick={on_rotate}
                                        >
                                            <i class="bi bi-arrow-repeat me-2"></i>
                                            {"Rotate"}
                                        </button>
                                        <button 
                                            class="btn btn-danger"
                                            onclick={on_revoke}
                                        >
                                            <i class="bi bi-x-circle me-2"></i>
                                            {"Revoke"}
                                        </button>
                                    </div>
                                }
                            </div>
                        </div>
                        <div class="pt-3 border-top">
                            <small class="text-muted d-flex gap-3">
                                <span>
                                    <i class="bi bi-calendar me-2"></i>
                                    {"Created: "}{format_date(key_data.created_at)}
                                </span>
                                if key_data.updated_at.is_some() {
                                    <span>
                                        <i class="bi bi-calendar me-2"></i>
                                        {"Updated: "}{format_date(key_data.updated_at)}
                                    </span>
                                }
                            </small>
                        </div>
                    </div>
                </div>
            }

            if !(*error_message).is_empty() {
                <div class="alert alert-danger mt-3">
                    {(*error_message).clone()}
                </div>
            }
        </div>
    }
}
