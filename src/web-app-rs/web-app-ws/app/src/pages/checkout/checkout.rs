use super::*;

#[cfg(feature = "ssr")]
use basket_ordering::basket_state::service::{BasketCheckoutInfo, BasketStateServiceContext};

use error_template::ErrorTemplate;

use leptos_meta::Title;
use leptos_router::hooks::use_navigate;
use stylers::style_sheet;
use uuid::Uuid;

#[cfg(feature = "ssr")]
use valitron::{
    available::{Message, Required, Trim},
    register::string::Validator,
    rule::string::StringRuleExt,
};

use crate::edit_form::*;
use field::*;

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BasketCheckoutInfoFormData {
    street: String,
    city: String,
    state: String,
    country: String,
    zip_code: String,
    request_id: Uuid,
}

impl BasketCheckoutInfoFormData {
    fn set_street(&self, value: &str) -> Self {
        let mut tmp = self.clone();
        tmp.street = value.to_string();
        tmp
    }
    fn get_street(&self) -> &str {
        &self.street
    }
    fn set_city(&self, value: &str) -> Self {
        let mut tmp = self.clone();
        tmp.city = value.to_string();
        tmp
    }
    fn get_city(&self) -> &str {
        &self.city
    }
    fn set_state(&self, value: &str) -> Self {
        let mut tmp = self.clone();
        tmp.state = value.to_string();
        tmp
    }
    fn get_state(&self) -> &str {
        &self.state
    }
    fn set_country(&self, value: &str) -> Self {
        let mut tmp = self.clone();
        tmp.country = value.to_string();
        tmp
    }
    fn get_country(&self) -> &str {
        &self.country
    }
    fn set_zip_code(&self, value: &str) -> Self {
        let mut tmp = self.clone();
        tmp.zip_code = value.to_string();
        tmp
    }
    fn get_zip_code(&self) -> &str {
        &self.zip_code
    }
}

#[cfg(feature = "ssr")]
impl BasketCheckoutInfoFormData {
    fn from_raw(input: &mut BasketCheckoutInfoFormData) -> Result<Self, Validator<Message>> {
        let valid = Validator::new()
            .insert("street", &mut input.street, Trim.and(Required))
            .insert("city", &mut input.city, Trim.and(Required))
            .insert("state", &mut input.state, Trim.and(Required))
            .insert("country", &mut input.country, Trim.and(Required))
            .insert("zip_code", &mut input.zip_code, Trim.and(Required));

        valid.validate(input.clone())
    }
}

#[server]
#[middleware(auth::RequireAuth)]
async fn get_basket_checkout_info_form_data() -> Result<BasketCheckoutInfoFormData, crate::AppError> {
    use auth::service::AuthServiceContext;
    let user_address_info = expect_context::<AuthServiceContext>().service.get_user_address_info().await?;
    //log!("user_address_info: {:#?}", user_address_info);

    Ok(BasketCheckoutInfoFormData {
        street: user_address_info.street.unwrap_or_default(),
        city: user_address_info.city.unwrap_or_default(),
        state: user_address_info.state.unwrap_or_default(),
        country: user_address_info.country.unwrap_or_default(),
        zip_code: user_address_info.zip.unwrap_or_default(),
        request_id: Uuid::new_v4(),
    })
}

#[server]
#[middleware(auth::RequireAuth)]
async fn submit_basket_checkout_info_form_data(data: BasketCheckoutInfoFormData) -> Result<(BasketCheckoutInfoFormData, Option<FormErrors>), crate::AppError> {
    let mut data = data.clone();
    let validation_result = BasketCheckoutInfoFormData::from_raw(&mut data);

    match validation_result {
        Ok(data) => {
            let basket_service = expect_context::<BasketStateServiceContext>().service;

            let data_ = data.clone();
            let card_expiration = chrono::Utc::now().checked_add_months(chrono::Months::new(12)).ok_or(crate::AppError::internal_server_error())?;

            let checkout_info = BasketCheckoutInfo {
                street: data.street,
                city: data.city,
                state: data.state,
                country: data.country,
                zip_code: data.zip_code,
                card_number: None,
                card_holder_name: None,
                card_security_number: None,
                card_expiration,
                card_type_id: 1,
                buyer: None,
                request_id: data.request_id,
            };
            basket_service.checkout(checkout_info).await?;

            Ok((data_, None))
        }
        Err(e) => Ok((data, Some(to_form_errors(e)))),
    }
}

#[component]
pub fn CheckoutPage() -> impl IntoView {
    let class_name = style_sheet!("./app/src/pages/checkout/checkout.css");

    let init_data_res = Resource::new(|| (), |_| async move { get_basket_checkout_info_form_data().await });

    let form_action = Action::new(move |data: &BasketCheckoutInfoFormData| {
        let data = data.clone();
        async move { submit_basket_checkout_info_form_data(data.clone()).await }
    });

    let checkout_view = move || {
        Suspend::new(async move {
            view! { class=class_name,
                <div class="checkout">
                    <EditForm
                        init_data_resource=init_data_res
                        form_action=form_action
                        on_ok=|_data| {
                            let navigate = use_navigate();
                            navigate("/user/orders", Default::default());
                        }
                    >
                        <div class="form">
                            <div class="form-section">
                                <h2>Shipping address</h2>
                                <label>
                                    Address
                                    <InPutText
                                        name=field!(street @ BasketCheckoutInfoFormData)
                                        setter=BasketCheckoutInfoFormData::set_street
                                        getter=BasketCheckoutInfoFormData::get_street
                                    />
                                    <ValidationMessage<
                                    BasketCheckoutInfoFormData,
                                >
                                        _phantom=std::marker::PhantomData
                                        name=field!(street @ BasketCheckoutInfoFormData)
                                    />
                                </label>
                                <div class="form-group">
                                    <div class="form-group-item">
                                        <label>
                                            City
                                            <InPutText
                                                name=field!(city @ BasketCheckoutInfoFormData)
                                                setter=BasketCheckoutInfoFormData::set_city
                                                getter=BasketCheckoutInfoFormData::get_city
                                            />
                                            <ValidationMessage<
                                            BasketCheckoutInfoFormData,
                                        >
                                                _phantom=std::marker::PhantomData
                                                name=field!(city @ BasketCheckoutInfoFormData)
                                            />
                                        </label>
                                    </div>
                                    <div class="form-group-item">
                                        <label>
                                            State
                                            <InPutText
                                                name=field!(state @ BasketCheckoutInfoFormData)
                                                setter=BasketCheckoutInfoFormData::set_state
                                                getter=BasketCheckoutInfoFormData::get_state
                                            />
                                            <ValidationMessage<
                                            BasketCheckoutInfoFormData,
                                        >
                                                _phantom=std::marker::PhantomData
                                                name=field!(state @ BasketCheckoutInfoFormData)
                                            />
                                        </label>
                                    </div>
                                    <div class="form-group-item">
                                        <label>
                                            Zip code
                                            <InPutText
                                                name=field!(zip_code @ BasketCheckoutInfoFormData)
                                                setter=BasketCheckoutInfoFormData::set_zip_code
                                                getter=BasketCheckoutInfoFormData::get_zip_code
                                            />
                                            <ValidationMessage<
                                            BasketCheckoutInfoFormData,
                                        >
                                                _phantom=std::marker::PhantomData
                                                name=field!(zip_code @ BasketCheckoutInfoFormData)
                                            />
                                        </label>
                                    </div>
                                </div>
                                <div>
                                    <label>
                                        Country
                                        <InPutText
                                            name=field!(country @ BasketCheckoutInfoFormData)
                                            setter=BasketCheckoutInfoFormData::set_country
                                            getter=BasketCheckoutInfoFormData::get_country
                                        />
                                        <ValidationMessage<
                                        BasketCheckoutInfoFormData,
                                    >
                                            _phantom=std::marker::PhantomData
                                            name=field!(country @ BasketCheckoutInfoFormData)
                                        />
                                    </label>
                                </div>
                            </div>
                            <div class="form-section">
                                <div class="form-buttons">
                                    <a href="cart" class="button button-secondary">
                                        <img role="presentation" src="icons/arrow-left.svg" />
                                        Back to the shopping bag
                                    </a>
                                    <button class="button button-primary" type="submit">
                                        Place order
                                    </button>
                                </div>
                            </div>
                        </div>
                        <ValidationSummary<
                        BasketCheckoutInfoFormData,
                    > _phantom=std::marker::PhantomData />
                    </EditForm>
                </div>
            }
        })
    };

    crate::app::page_header::set_title("Checkout");
    view! { class=class_name,
        <Title text=format!("Checkout | AdventureWorks") />
        <Transition fallback=move || view! { <p>"Loading data..."</p> }>
            <ErrorBoundary fallback=|errors| {
                view! { <ErrorTemplate errors /> }
            }>

                {checkout_view}

            </ErrorBoundary>
        </Transition>
    }
}
