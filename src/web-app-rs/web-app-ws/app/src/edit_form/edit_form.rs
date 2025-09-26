use leptos::{prelude::*, reactive::spawn_local};

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[cfg(feature = "ssr")]
use valitron::{available::Message, register::string::Validator};

use super::Field;
use super::FormField;

pub type FormErrors = HashMap<String, Vec<String>>;

#[cfg(feature = "ssr")]
pub fn to_form_errors(value: Validator<Message>) -> FormErrors {
    let mut map = HashMap::new();
    for (key, value) in value.into_iter() {
        let value = value.into_iter().map(|v| v.to_string()).collect();
        map.insert(key, value);
    }
    map
}

#[derive(Clone)]
pub struct FormState<T, F>
where
    T: Sync + Clone + Send + 'static,
    F: FormField<T>,
{
    pub form_data_validated: RwSignal<(T, FormErrors)>,
    pub form_data_current: RwSignal<T>,
    pub fields: Arc<Mutex<Vec<F>>>,
}

#[component]
pub fn EditForm<T>(init_data_resource: Resource<Result<T, crate::AppError>>, children: Children, form_action: Action<T, Result<(T, Option<FormErrors>), crate::AppError>>, on_ok: fn(T)) -> impl IntoView
where
    T: Default + Sync + Clone + Send + 'static,
{
    let form_data_validated = RwSignal::new((T::default(), FormErrors::default()));
    let form_data_current = RwSignal::new(T::default());
    let errors = RwSignal::new(Vec::<crate::AppError>::default());
    Effect::new(move || {
        spawn_local(async move {
            match init_data_resource.await {
                Ok(data) => {
                    form_data_validated.set((data.clone(), FormErrors::default()));
                    form_data_current.set(data.clone());
                }
                Err(e) => {
                    errors.set(vec![e]);
                }
            }
        });
    });

    let submit = Action::new(move |_: &()| {
        let form_data = form_data_current;
        async move {
            form_action.dispatch(form_data.get_untracked());
            Effect::new(move || {
                if let Some(res) = form_action.value().get() {
                    match res {
                        Ok((data, Some(errors))) => {
                            form_data_validated.set((data.clone(), errors));
                            form_data_current.set(data);
                        }
                        Ok((data, None)) => {
                            on_ok(data);
                        }
                        Err(e) => {
                            errors.set(vec![e]);
                        }
                    }
                };
            });
        }
    });

    // Provide context to children
    provide_context(FormState::<T, Field<T>> {
        form_data_validated,
        form_data_current,
        fields: Arc::new(Mutex::new(Vec::<Field<T>>::new())),
    });

    view! {
        <form on:submit=move |ev| {
            ev.prevent_default();
            submit.dispatch(());
        }>{children()}</form>
    }
}
