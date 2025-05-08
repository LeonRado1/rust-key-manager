use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use yew_router::prelude::Link;
use crate::components::app_router::Route;
use crate::constants::key_types::get_type_class;
use crate::context::user_context::use_user_context;
use crate::helpers::date::format_date;
use crate::helpers::storage;
use crate::models::key::PartialKey;
use crate::services::keys;

#[function_component(Expired)]
pub fn expired() -> Html {

    let keys = use_state(Vec::<PartialKey>::new);

    let user_ctx = use_user_context();
    let navigator = use_navigator().unwrap();

    {
        let keys = keys.clone();

        let user_ctx = user_ctx.clone();
        let navigator = navigator.clone();

        use_effect_with(user_ctx.clone(), move |ctx| {
            if ctx.user.is_none() {
                navigator.push(&Route::Login);
            }
            else {
                spawn_local(async move {
                    if let Ok(token) = storage::get_token() {
                        match keys::get_expired_keys(&token).await {
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
    
    html! {
        <main>
            <div>
                <h3 class="m-0">
                   {"Expired Keys"}
                </h3>
            </div>
            if !keys.is_empty() {
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
                        {(*keys).clone().into_iter()
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
                    {"You don't have any expired keys yet. Your expired keys will be listed here."}
                </div>
            }
        </main>
    }
}
