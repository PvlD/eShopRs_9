use leptos::prelude::RwSignal;
pub trait FormField<FormData: Send + Clone + 'static> {
    fn update_value(&self, data: &FormData) -> FormData;
    fn value_validated_signal(&self) -> RwSignal<Option<Vec<String>>>;
    fn name(&self) -> &'static str;
}

#[derive(Clone)]
pub enum Field<FormData: Send + Clone + 'static> {
    TextFormField(super::FormFieldText<FormData>),
}

impl<FormData: Send + Clone + 'static> FormField<FormData> for Field<FormData> {
    fn update_value(&self, data: &FormData) -> FormData {
        match self {
            Field::TextFormField(field) => field.update_value(data),
        }
    }
    fn value_validated_signal(&self) -> RwSignal<Option<Vec<String>>> {
        match self {
            Field::TextFormField(field) => field.value_validated_signal(),
        }
    }
    fn name(&self) -> &'static str {
        match self {
            Field::TextFormField(field) => field.name(),
        }
    }
}
