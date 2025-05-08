use web_sys::HtmlInputElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use yew_router::prelude::Link;
use crate::components::app_router::Route;
use crate::constants::key_types::{get_type_class, API_KEY, PASSWORD, SSH_KEY, TOKEN};
use crate::context::user_context::use_user_context;
use crate::helpers::date::format_date;
use crate::helpers::storage;
use crate::models::key::PartialKey;
use crate::services::keys;

#[function_component(Dashboard)]
pub fn dashboard() -> Html {

    let keys = use_state(Vec::<PartialKey>::new);

    let active_filter = use_state(|| None::<i32>);
    let search_text = use_state(|| String::new());

    let user_ctx = use_user_context();
    let navigator = use_navigator().unwrap();

    {
        let keys = keys.clone();

        let user_ctx = user_ctx.clone();
        let navigator = navigator.clone();

        use_effect_with(user_ctx.clone(), move |ctx| {
            if !ctx.is_loading && ctx.user.is_none() {
                navigator.push(&Route::Login);
            }
            else {
                spawn_local(async move {
                    if let Ok(token) = storage::get_token() {
                        match keys::get_keys(&token).await {
                            Ok(response) => {
                                keys.set(response);
                            }
                            Err(_e) => ()
                        }
                    }
                });
            }
            || ()
        });
    }

    let password_count = keys.iter().filter(|k| k.key_type_id == PASSWORD).count();
    let token_count = keys.iter().filter(|k| k.key_type_id == TOKEN).count();
    let api_key_count = keys.iter().filter(|k| k.key_type_id == API_KEY).count();
    let ssh_key_count = keys.iter().filter(|k| k.key_type_id == SSH_KEY).count();

    let filtered_keys: Vec<&PartialKey> = keys.iter()
        .filter(|key| match *active_filter {
            Some(filter) => key.key_type_id == filter,
            None => true
        })
        .filter(|key| {
            let search = (*search_text).as_str();

            if search.is_empty() {
                return true;
            }

            key.key_name.to_lowercase().contains(search) ||
            key.key_description.as_ref()
                .map_or(false, |d| d.to_lowercase().contains(search)) ||
            key.key_tag.as_ref()
                .map_or(false, |t| t.to_lowercase().contains(search))
        })
        .collect();

    let on_password_click = {
        let active_filter = active_filter.clone();

        Callback::from(move |_e: MouseEvent| {
            active_filter.set(match *active_filter {
                Some(PASSWORD) => None,
                _ => Some(PASSWORD)
            });
        })
    };

    let on_token_click = {
        let active_filter = active_filter.clone();
        Callback::from(move |_e: MouseEvent| {
            active_filter.set(match *active_filter {
                Some(TOKEN) => None,
                _ => Some(TOKEN)
            });
        })
    };

    let on_api_key_click = {
        let active_filter = active_filter.clone();
        Callback::from(move |_e: MouseEvent| {
            active_filter.set(match *active_filter {
                Some(API_KEY) => None,
                _ => Some(API_KEY)
            });
        })
    };

    let on_ssh_key_click = {
        let active_filter = active_filter.clone();
        Callback::from(move |_e: MouseEvent| {
            active_filter.set(match *active_filter {
                Some(SSH_KEY) => None,
                _ => Some(SSH_KEY)
            });
        })
    };

    let on_search_change = {
        let search_text = search_text.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            search_text.set(input.value().to_lowercase());
        })
    };

    html! {
        <main>
            <div class="d-flex align-items-center justify-content-between">
                <h1 class="m-0">
                   {"Dashboard"}
                </h1>
                <div class="d-flex gap-2">
                    <span class="badge text-bg-success fw-normal">
                        <span class="fw-bold me-1">
                            {password_count}
                        </span>
                        {"Passwords"}
                    </span>
                    <span class="badge text-bg-danger fw-normal">
                        <span class="fw-bold me-1">
                            {token_count}
                        </span>
                        {"Tokens"}
                    </span>
                    <span class="badge text-bg-warning fw-normal">
                        <span class="fw-bold me-1">
                            {api_key_count}
                        </span>
                        {"API Keys"}
                    </span>
                    <span class="badge text-bg-info fw-normal">
                        <span class="fw-bold me-1">
                            {ssh_key_count}
                        </span>
                        {"SSH Keys"}
                    </span>
                </div>
            </div>
            if !keys.is_empty() {
                <div class="d-flex align-items-center justify-content-between mt-5">
                    <div class="d-flex col-10 align-items-center gap-5">
                        <input
                            class="form-control w-25"
                            type="text"
                            placeholder="Search"
                            value={(*search_text).clone()}
                            oninput={on_search_change}
                        />
                        <div class="btn-group">
                            <button
                                onclick={on_password_click}
                                class={classes!("btn", "btn-outline-dark",
                                    (*active_filter == Some(PASSWORD)).then_some("active"))}
                            >
                                <i class="bi bi-key-fill me-1"></i>
                                {"Passwords"}
                            </button>
                            <button
                                onclick={on_token_click}
                                class={classes!("btn", "btn-outline-dark",
                                    (*active_filter == Some(TOKEN)).then_some("active"))}
                            >
                                <i class="bi bi-lock-fill me-1"></i>
                                {"Tokens"}
                            </button>
                            <button
                                onclick={on_api_key_click}
                                class={classes!("btn", "btn-outline-dark",
                                    (*active_filter == Some(API_KEY)).then_some("active"))}
                            >
                                <i class="bi bi-database-fill me-1"></i>
                                {"API Keys"}
                            </button>
                            <button
                                onclick={on_ssh_key_click}
                                class={classes!("btn", "btn-outline-dark",
                                    (*active_filter == Some(SSH_KEY)).then_some("active"))}
                            >
                                <i class="bi bi-hdd-stack-fill me-1"></i>
                                {"SSH Keys"}
                            </button>
                        </div>
                    </div>
                    <div class="d-flex col-2 align-items-center justify-content-end gap-3">
                        <Link<Route>
                            to={Route::Revoked}
                            classes="btn btn-sm btn-link p-0 text-secondary"
                        >
                            <i class="bi bi-x-circle me-1"></i>
                            {"Revoked"}
                        </Link<Route>>
                        <Link<Route>
                            to={Route::Expired}
                            classes="btn btn-sm btn-link p-0 text-secondary"
                        >
                            <i class="bi bi-clock me-1"></i>
                            {"Expired"}
                        </Link<Route>>
                    </div>
                </div>
                if !filtered_keys.is_empty() {
                    <table class="table table-responsive table-hover mt-3">
                        <thead>
                            <tr>
                                <th scope="col">{"Type"}</th>
                                <th scope="col">{"Name"}</th>
                                <th scope="col">{"Description"}</th>
                                <th scope="col">{"Tag"}</th>
                                <th scope="col">{"Expiration"}</th>
                                <th scope="col"></th>
                            </tr>
                        </thead>
                        <tbody class="table-group-divider">
                            {filtered_keys.into_iter()
                                .map(|key| {
                                    html! {
                                        <tr key={key.id.to_string()}>
                                            <td>
                                                <span class={classes!("badge", get_type_class(key.key_type_id))}>
                                                    {&key.key_type}
                                                </span>
                                            </td>
                                            <td>{&key.key_name}</td>
                                            <td>
                                                {
                                                    key.key_description
                                                        .as_deref()
                                                        .map_or("".to_string(), |desc| {
                                                            if desc.len() > 40 {
                                                                format!("{}...", &desc[..37])
                                                            } else {
                                                                desc.to_string()
                                                            }
                                                        })
                                                }
                                            </td>
                                            <td>
                                                <span class="badge rounded-pill text-bg-dark">
                                                    {key.key_tag.as_deref().unwrap_or("")}
                                                </span>
                                            </td>
                                            <td>
                                                {
                                                    key.expiration_date
                                                        .map_or(
                                                            String::new(),
                                                            |d| format_date(Some(d))
                                                        )
                                                }
                                            </td>
                                            <td>
                                                <Link<Route>
                                                    to={Route::KeyDetail { id: key.id } }
                                                    classes="btn btn-sm btn-link p-0 text-decoration-none"
                                                >
                                                    <i class="bi bi-box-arrow-up-right me-1"></i>
                                                    {"Detail"}
                                                </Link<Route>>
                                            </td>
                                        </tr>
                                    }
                            }).collect::<Html>()}
                        </tbody>
                    </table>
                }
                else {
                    <div class="alert alert-light mt-3" role="alert">
                        {"No keys match your search and filter criteria."}
                    </div>
                }
            }
            else {
                <div class="alert alert-light mt-5" role="alert">
                    {"You don't have any keys yet. Your keys will be listed here."}
                </div>
            }
        </main>
    }
}
