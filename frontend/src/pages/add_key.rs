use web_sys::{File, HtmlInputElement};
use yew::prelude::*;
use crate::constants::key_types::{PASSWORD, SSH_KEY, get_type_name, get_type_class, TOKEN};
use crate::helpers::string_utils::{generate_password, string_or_none};
use crate::models::key::KeyRequest;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub id: i32,
}

#[function_component(AddKey)]
pub fn add_key(props: &Props) -> Html {

    let key_request = use_state(KeyRequest::default);
    let private_key_file = use_state(|| None::<File>);

    let show_password = use_state(|| false);
    let file_input_ref = use_node_ref();

    let on_name_change = {
        let key_request = key_request.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_request = (*key_request).clone();
            new_request.key_name = input.value();
            key_request.set(new_request);
        })
    };

    let on_description_change = {
        let key_request = key_request.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_request = (*key_request).clone();
            new_request.key_description = string_or_none(input.value());
            key_request.set(new_request);
        })
    };

    let on_tag_change = {
        let key_request = key_request.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_request = (*key_request).clone();
            new_request.key_tag = string_or_none(input.value());
            key_request.set(new_request);
        })
    };


    let on_value_change = {
        let key_request = key_request.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_request = (*key_request).clone();
            new_request.key_value = string_or_none(input.value());
            key_request.set(new_request);
        })
    };

    let on_expiration_change = {
        let key_request = key_request.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_request = (*key_request).clone();
            new_request.expiration_date = string_or_none(input.value());
            key_request.set(new_request);
        })
    };

    let toggle_password_visibility = {
        let show_password = show_password.clone();
        Callback::from(move |_| {
            show_password.set(!*show_password);
        })
    };

    let generate_value = {
        let key_request = key_request.clone();
        Callback::from(move |_| {
            let generated = generate_password(16);
            let mut new_request = (*key_request).clone();
            new_request.key_value = Some(generated.clone());
            key_request.set(new_request);
        })
    };

    let on_public_key_change = {
        let key_request = key_request.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let mut new_request = (*key_request).clone();
            new_request.key_value = string_or_none(input.value());
            key_request.set(new_request);
        })
    };

    let on_private_key_change = {
        let private_key_file = private_key_file.clone();
        let file_input_ref = file_input_ref.clone();

        Callback::from(move |_| {
            if let Some(input) = file_input_ref.cast::<HtmlInputElement>() {
                if let Some(files) = input.files() {
                    if let Some(file) = files.get(0) {
                        private_key_file.set(Some(file));
                    }
                    else {
                        private_key_file.set(None);
                    }
                }
            }
        })
    };

    let is_save_valid = || -> bool {
        let key_request = key_request.clone();

        match props.id {
            PASSWORD => {
                !key_request.key_name.is_empty() && !key_request.key_value.is_none()
            },
            _ => !key_request.key_name.is_empty()
        }
    };

    let is_import_valid = || -> bool {
        let key_request = key_request.clone();
        let private_key_file = private_key_file.clone();

        !key_request.key_name.is_empty() && key_request.key_value.is_some() && private_key_file.is_some()
    };

    html! {
        <main>
            <div class="d-flex align-items-center justify-content-between mb-3">
                <h1 class="m-0">
                    {"Add Key"}
                </h1>
                <h5 class="m-0">
                    <span class={classes!("badge", format!("text-{}", &get_type_class(props.id)))}>
                        {get_type_name(props.id)}
                    </span>
                </h5>
            </div>
            <form>
                <div class="row">
                    <div class="col-md-6">
                        <div class="mb-3">
                            <label class="form-label">{"Name*"}</label>
                            <input
                                type="text"
                                class="form-control"
                                value={key_request.key_name.clone()}
                                oninput={on_name_change}
                            />
                        </div>
                        <div class="mb-3">
                            <label class="form-label">{"Description"}</label>
                            <input
                                type="text"
                                class="form-control"
                                value={key_request.key_description.clone().unwrap_or_default()}
                                oninput={on_description_change}
                            />
                        </div>
                        <div class="mb-3">
                            <label class="form-label">{"Tag"}</label>
                            <input
                                type="text"
                                class="form-control"
                                value={key_request.key_tag.clone().unwrap_or_default()}
                                oninput={on_tag_change}
                            />
                        </div>
                    </div>
                    <div class="col-md-6">
                        <div class="mb-3">
                            <label class="form-label">{"Value*"}</label>
                            <div class="input-group">
                                <input
                                    type={if *show_password { "text" } else { "password" }}
                                    class="form-control"
                                    value={if props.id == PASSWORD { key_request.key_value.clone() } else { None }}
                                    oninput={on_value_change}
                                    placeholder={if props.id != PASSWORD { "Key will be generated automatically" } else { "" }}
                                    disabled={props.id != PASSWORD}
                                />
                                if props.id == PASSWORD {
                                    <button
                                        type="button"
                                        class="btn btn-outline-danger"
                                        onclick={toggle_password_visibility}
                                    >
                                        if *show_password {
                                            <i class="bi bi-eye-slash"></i>
                                        }
                                        else {
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
                                }
                            </div>
                        </div>
                        <div class="mb-3">
                            <label class="form-label">{"Expiration Date"}</label>
                            <div class="input-group">
                                <input
                                    type="text"
                                    class="form-control"
                                    value={key_request.expiration_date.clone().unwrap_or_default()}
                                    placeholder={
                                        if props.id != TOKEN { "Expiration date is not for this key type" } else { "YY/MM/DD hh:mm:ss" }
                                    }
                                    disabled={props.id != TOKEN}
                                    oninput={on_expiration_change}
                                />
                                <span class="input-group-text">
                                    <i class="bi bi-calendar2-event"></i>
                                </span>
                            </div>
                        </div>
                    </div>
                </div>

                if props.id == SSH_KEY {
                    <div class="row mt-3">
                        <div class="col-12">
                            <h4>{"SSH Key Import"}</h4>
                        </div>
                        <div class="col-md-6">
                            <div class="mb-3">
                                <label class="form-label">{"Public Key"}</label>
                                <textarea
                                    class="form-control"
                                    rows="4"
                                    style="resize: none"
                                    value={key_request.key_value.clone().unwrap_or_default()}
                                    oninput={on_public_key_change}
                                />
                            </div>
                        </div>
                        <div class="col-md-6">
                            <div class="mb-3">
                                <label class="form-label">{"Private Key File"}</label>
                                <input
                                    type="file"
                                    accept=".pem"
                                    class="form-control"
                                    ref={file_input_ref}
                                    onchange={on_private_key_change}
                                />
                            </div>
                        </div>
                    </div>
                    <div class="alert alert-warning">
                        {"If you choose to import the key, the value field will be ignored."}
                    </div>
                }
                if props.id == TOKEN {
                    <div class="alert alert-info alert-dismissible fade show">
                        {"If you enter expiration date, you will be notified before expiration."}
                    </div>
                }
                <div class="alert alert-danger">
                    {"Alert message if needed."}
                </div>
                <div class="row my-4">
                    <div class="col-12">
                        <button
                            type="button"
                            class="btn btn-outline-primary"
                            disabled={!is_save_valid()}
                        >
                            <i class="bi bi-floppy2-fill me-1"></i>
                            {"Save"}
                        </button>
                        if props.id == SSH_KEY {
                            <button
                                type="button"
                                class="btn btn-outline-success ms-3"
                                disabled={!is_import_valid()}
                            >
                                <i class="bi bi-cloud-upload-fill me-1"></i>
                                {"Import"}
                            </button>
                        }
                    </div>
                </div>
            </form>
        </main>
    }
}
