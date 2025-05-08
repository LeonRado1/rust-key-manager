use regex::Regex;
use yew::prelude::*;

#[function_component(ChangePwd)]
pub fn change_pwd() -> Html {
    let error_message = use_state(|| String::new());

    pub fn is_email_valid(email: &str) -> bool {
        match Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$") {
            Ok(email_regex) => email_regex.is_match(email),
            Err(_) => false
        }
    }

    html! {
         <div class="col-lg-4 mx-auto">
            <h2 class="text-center mb-3">{ "Change Password" }</h2>
            <form>
                <div class="mb-3">
                    <label class="form-label" for="email">{ "Email" }</label>
                    <input
                        id="email"
                        type="email"
                        class="form-control"
                    />
                </div>
                <div class="mb-4">
                    <label class="form-label" for="password">{ "New Password" }</label>
                    <input
                        id="password"
                        type="password"
                        class="form-control"
                    />
                </div>
                <div class="pt-3 border-top">
                    <label class="form-label" for="code">{ "Recovery Code" }</label>
                    <input
                        id="code"
                        type="text"
                        class="form-control"
                    />
                </div>
                <div class="text-center mt-4">
                    <button
                        class="btn btn-outline-primary mx-auto w-50"
                        type="button"
                    >
                        <i class="bi bi-arrow-up-right-square-fill me-2"></i>
                        { "Change" }
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
