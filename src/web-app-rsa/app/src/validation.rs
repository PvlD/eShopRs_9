use garde::Validate;
use leptos::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub struct ValidationErrors {
    errors: RwSignal<HashMap<String, Vec<String>>>,
}

impl ValidationErrors {
    pub fn new() -> Self {
        Self {
            errors: RwSignal::new(HashMap::new()),
        }
    }

    pub fn clear(&self) {
        self.errors.set(HashMap::new());
    }

    pub fn get(&self, field: &str) -> Vec<String> {
        self.errors
            .get()
            .get(field)
            .cloned()
            .unwrap_or_default()
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.get().is_empty()
    }

    pub fn has_field_error(&self, field: &str) -> bool {
        self.errors
            .get()
            .get(field)
            .map_or(false, |e| !e.is_empty())
    }

    pub fn add(&self, field: &str, message: &str) {
        self.errors.update(|m| {
            m.entry(field.to_string())
                .or_insert_with(Vec::new)
                .push(message.to_string());
        });
    }

    pub fn all_errors(&self) -> Vec<(String, Vec<String>)> {
        let mut result: Vec<_> = self
            .errors
            .get()
            .into_iter()
            .collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    pub fn validate<T: Validate>(
        &self,
        model: &T,
        label_fn: fn(&str) -> &'static str,
    ) -> bool
    where
        T::Context: Default,
    {
        self.clear();
        if let Err(report) = model.validate() {
            let mut map = HashMap::new();
            for (path, error) in report.iter() {
                let field = path.to_string();
                let label = label_fn(&field);
                let msg = format!("The {} field {}", label, error);
                map.entry(field)
                    .or_insert_with(Vec::new)
                    .push(msg);
            }
            self.errors.set(map);
            false
        } else {
            true
        }
    }
}

impl Default for ValidationErrors {
    fn default() -> Self {
        Self::new()
    }
}

#[component]
pub fn ValidationMessage(
    field: &'static str,
    errors: ValidationErrors,
) -> impl IntoView {
    let msg = move || {
        let msgs = errors.get(field);
        msgs.first().cloned().unwrap_or_default()
    };

    move || {
        let m = msg();
        (!m.is_empty()).then(|| {
            view! { <p class="validation-message">{m}</p> }
        })
    }
}

#[component]
pub fn ValidationSummary(
    errors: ValidationErrors,
) -> impl IntoView {
    let items = move || errors.all_errors();

    move || {
        let all = items();
        if all.is_empty() {
            return None;
        }
        Some(view! {
            <ul class="validation-errors">
                {all.into_iter().flat_map(|(_, msgs)| {
                    msgs.into_iter().map(|msg| {
                        view! { <li class="validation-message">{msg}</li> }
                    })
                }).collect::<Vec<_>>()}
            </ul>
        })
    }
}
