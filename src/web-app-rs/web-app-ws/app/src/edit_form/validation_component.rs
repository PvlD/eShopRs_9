use super::Field;
use super::FormField;
use super::FormState;
use leptos::{component, prelude::*};

#[component]
pub fn ValidationMessage<T>(name: &'static str, _phantom: std::marker::PhantomData<T>) -> impl IntoView
where
    T: Sync + Send + Clone + 'static,
{
    // Get the form state from context
    let form_state = expect_context::<FormState<T, Field<T>>>();

    let validated_sig = form_state.fields.lock().unwrap().iter().find(|f| f.name() == name).unwrap().value_validated_signal();

    // Get error for this field (if any)
    let err_view = move || match validated_sig.get() {
        Some(errors_) => errors_.into_iter().map(|e| view! { <div class="validation-message">{e.to_string()}</div> }).collect::<Vec<_>>().into_any(),
        None => {
            let _: () = view! {};
            ().into_any()
        }
    };

    view! { {err_view} }
}

#[component]
pub fn ValidationSummary<T>(_phantom: std::marker::PhantomData<T>) -> impl IntoView
where
    T: Sync + Send + Clone + 'static,
{
    // Get the form state from context
    let form_state = expect_context::<FormState<T, Field<T>>>();

    // Get error for this field (if any)
    let err_view = move || match form_state.form_data_validated.get().1 {
        errors if !errors.is_empty() => view! {
            <ul class="validation-errors">
                {errors
                    .iter()
                    .map(|(k, v)| {
                        v.iter()
                            .map(|v| {
                                view! {
                                    <li class="validation-message">
                                        {k.to_string()}: {v.to_string()}
                                    </li>
                                }
                            })
                            .collect::<Vec<_>>()
                            .into_any()
                    })
                    .collect::<Vec<_>>()
                    .into_any()}
            </ul>
        }
        .into_any(),
        _ => {
            let _: () = view! {};
            ().into_any()
        }
    };

    view! { {err_view} }
}
