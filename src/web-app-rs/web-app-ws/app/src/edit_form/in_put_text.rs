use leptos::{html::Input, prelude::*};

use super::{Field, FormField, FormFieldText};

#[component]
pub fn InPutText<T: Sync + Send + Clone + 'static + std::fmt::Debug>(name: &'static str, setter: fn(&T, &str) -> T, getter: fn(&T) -> &str) -> impl IntoView {
    // Get the form state from context
    let form_state = expect_context::<super::FormState<T, Field<T>>>();

    let input_ref = NodeRef::<Input>::new();

    let field_cb = Field::TextFormField(FormFieldText::new(name, setter, RwSignal::new(None), input_ref));

    let validated_sig = field_cb.value_validated_signal();

    let value_sig_ = Signal::derive(move || {
        let (v, e) = form_state.form_data_validated.get();
        let value = getter(&v).to_string();
        let is_err = match e.get(name) {
            Some(errors) => {
                validated_sig.set(Some(errors.clone()));
                true
            }
            None => {
                validated_sig.set(None);
                false
            }
        };

        (value, is_err)
    });
    form_state.fields.lock().unwrap().push(field_cb.clone());

    let vf = move || {
        let field_cb = field_cb.clone();
        let (value, is_err) = value_sig_.get();
        view! {
            <input
                type="text"
                name=name
                class=if is_err { "invalid" } else { "" }
                aria-invalid=is_err.to_string()
                prop:value=value.clone()
                value=value
                node_ref=input_ref
                on:input=move |ev| {
                    ev.prevent_default();
                    let new_data = field_cb.update_value(&form_state.form_data_current.get());
                    form_state
                        .form_data_current
                        .update(|v| {
                            *v = new_data;
                        });
                }
            />
        }
    };

    view! { {vf} }
}
