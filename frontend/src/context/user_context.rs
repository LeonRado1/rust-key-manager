use std::rc::Rc;
use yew::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::models::user::User;
use crate::helpers::storage;
use crate::services::auth;

#[derive(Clone, Debug, PartialEq)]
pub struct UserContext {
    pub user: Option<User>,
    pub set_user: Callback<Option<User>>,
}

#[derive(Properties, PartialEq)]
pub struct ProviderProps {
    pub children: Children,
}

#[function_component(UserContextProvider)]
pub fn user_context_provider(props: &ProviderProps) -> Html {
    let user = use_state(|| None::<User>);

    let set_user = {
        let user = user.clone();
        Callback::from(move |new_user: Option<User>| {
            user.set(new_user);
        })
    };

    {
        let set_user = set_user.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(token) = storage::get_token() {
                    match auth::get_current_user(&token).await {
                        Ok(response) => {
                            if let Ok(_) = storage::save_token(&response.token) {
                                set_user.emit(Some(response.user));
                            }
                        }
                        Err(_e) => {
                            let _ = storage::remove_token();
                        }
                    }
                }
            });
        });
    }

    let context = UserContext {
        user: (*user).clone(),
        set_user,
    };

    html! {
        <ContextProvider<Rc<UserContext>> context={Rc::new(context)}>
            {props.children.clone()}
        </ContextProvider<Rc<UserContext>>>
    }
}

#[hook]
pub fn use_user_context() -> Rc<UserContext> {
    use_context::<Rc<UserContext>>().expect("UserContext not found")
}
