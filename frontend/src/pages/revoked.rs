use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::hooks::use_navigator;
use crate::components::app_router::Route;
use crate::constants::key_types::get_type_class;
use crate::context::user_context::use_user_context;
use crate::helpers::date::format_date;
use crate::helpers::storage;
use crate::models::key::PartialKey;
use crate::services::keys;

#[function_component(Revoked)]
pub fn revoked() -> Html {

    let keys = use_state(Vec::<PartialKey>::new);
    
    let success_message = use_state(|| String::new());
    let error_message = use_state(|| String::new());

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
                        match keys::get_revoked_keys(&token).await {
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

    let on_delete = {
        let keys = keys.clone();
        
        let success_message = success_message.clone();
        let error_message = error_message.clone();

        Callback::from(move |key_id: i32| {
            let keys = keys.clone();
            
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
                    match keys::delete_key(&token, key_id).await {
                        Ok(_) => {
                            let new_keys: Vec<PartialKey> = (*keys).clone().into_iter()
                                .filter(|key| key.id != key_id)
                                .collect();
                            
                            keys.set(new_keys);
                            success_message.set("Key has been permanently deleted.".to_string());
                        }
                        Err(err) => error_message.set(err),
                    }
                }
            });
        })
    };

    html! {
        <main>
            <div>
                <h3 class="m-0">
                   {"Revoked Keys"}
                </h3>
            </div>
            if !success_message.is_empty() {
                <div class="alert alert-success alert-dismissible fade show mt-3">
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
                                            <button 
                                                class="btn btn-sm btn-danger"
                                                onclick={
                                                    let on_delete = on_delete.clone();
                                                    let key_id = key.id;
                                                    move |_e: MouseEvent| on_delete.emit(key_id)
                                                }
                                            >
                                                <i class="bi bi-trash-fill"></i>
                                            </button>
                                        </td>
                                    </tr>
                                }
                        }).collect::<Html>()}
                    </tbody>
                </table>
            }
            else {
                <div class="alert alert-light mt-3" role="alert">
                    {"You didn't revoke any key yet. Your revoked keys will be listed here."}
                </div>
            }
            
            if !(*error_message).is_empty() {
                <div class="alert alert-danger mt-3">
                    {(*error_message).clone()}
                </div>
            }
        </main>
    }
}
