use leptos::{
    html::Input,
    prelude::{GetUntracked, NodeRef, RwSignal},
};

#[derive(Clone)]
pub struct FormFieldText<FormData: Send + Clone + 'static> {
    name: &'static str,
    value_setter: fn(&FormData, &str) -> FormData,
    validated: RwSignal<Option<Vec<String>>>,
    html_elm: NodeRef<Input>,
}

impl<FormData: Send + Clone + 'static> FormFieldText<FormData> {
    pub fn new(name: &'static str, value_setter: fn(&FormData, &str) -> FormData, validated: RwSignal<Option<Vec<String>>>, html_elm: NodeRef<Input>) -> FormFieldText<FormData> {
        FormFieldText { name, value_setter, validated, html_elm }
    }
}

impl<FormData: Send + Clone + 'static> FormFieldText<FormData> {
    pub fn update_value(&self, data: &FormData) -> FormData {
        let new_value = self.html_elm.get_untracked().unwrap().value();

        (self.value_setter)(data, &new_value)
    }
}

impl<FormData: Send + Clone + 'static> FormFieldText<FormData> {
    pub fn value_validated_signal(&self) -> RwSignal<Option<Vec<String>>> {
        self.validated
    }
}

impl<FormData: Send + Clone + 'static> FormFieldText<FormData> {
    pub fn name(&self) -> &'static str {
        self.name
    }
}
