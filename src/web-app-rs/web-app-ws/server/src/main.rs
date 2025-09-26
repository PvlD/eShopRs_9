mod forwarder;

use api_version::versioning;

use anyhow::Result;

use auth::{openid_client, users::Backend};
use log::{error, info};
use log4rs;

use time::Duration;
use url::Url;

pub(crate) use catalog::server::*;
pub(crate) use url_mapper;
mod eventbus;

#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    let r = main_().await;

    if let Err(e) = r {
        error!("{:?}", e);
    } else {
        info!("Done");
    }
}

async fn main_() -> Result<()> {
    use ::app::*;
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::LeptosRoutes;
    use leptos_axum::generate_route_list;

    if std::env::var("IdentityUrl").is_err() {
        // not from aspire
        match dotenvy::from_filename("webapp.env") {
            Ok(path) => println!(".env read successfully from {}", path.display()),
            Err(e) => println!(
                "Could not load .env file: {e}. \nProceeding assuming variables are set in the \
                 environment."
            ),
        };
    }

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    /*     let url_map = vec![
           ("http://basket-api", "http://localhost:5221"),
           ("http://catalog-api", "http://localhost:5222"),
           ("http://ordering-api", "http://localhost:5224"),
       ];
    */
    let eventbus = eventbus::init_eventbus("web_app_rs").await;

    let url_mapper = url_mapper::from_env();

    let catalog_service_context = ::catalog::server::make_service(HttpClient::new(), url_mapper.clone(), versioning::QueryStringApiVersion::from((1, 0)))?;

    let basket_service_context = basket_ordering::basket::server::make_service(url_mapper.clone()).await.unwrap();

    let ordering_service_context = basket_ordering::ordering::server::make_service(basket_ordering::ordering::server::HttpClient::new(), url_mapper.clone(), versioning::QueryStringApiVersion::from((1, 0))).await.unwrap();

    let basket_state_service_context = basket_ordering::basket_state::server::make_service(basket_service_context.clone(), catalog_service_context.clone(), ordering_service_context.clone()).unwrap();

    let auth_service_context = auth::server::make_service().unwrap();

    let site_url = Url::parse(format!("http://{}", leptos_options.site_addr).as_str()).unwrap();

    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .use_rustls_tls()
        .use_native_tls()
        .build()
        .unwrap();

    let openid_client = openid_client::create_from_env(http_client.clone(), site_url).await.unwrap();

    let db = sqlx::SqlitePool::connect(":memory:").await?;
    sqlx::migrate!().run(&db).await?;

    let session_store = axum_login::tower_sessions::MemoryStore::default();
    let session_layer = axum_login::tower_sessions::SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(axum_login::tower_sessions::cookie::SameSite::Lax) // Ensure we send the cookie from the OAuth redirect.
        .with_expiry(axum_login::tower_sessions::Expiry::OnInactivity(Duration::days(1)));

    // Auth service.
    //
    // This combines the session layer with our backend to establish the auth
    // service which will provide the auth session as a request extension.

    let backend = Backend::new(db, openid_client, http_client);
    let auth_layer = axum_login::AuthManagerLayerBuilder::new(backend, session_layer).build();

    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || {
                provide_context(catalog_service_context.clone());

                provide_context(basket_service_context.clone());
                provide_context(ordering_service_context.clone());
                provide_context(basket_state_service_context.clone());
                provide_context(auth_service_context.clone());

                //provide_context(product_image_url_context.clone());
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options)
        .layer(auth_layer);

    let forwarder_router = forwarder::product_images::router(url_mapper.clone());
    let app: Router = Router::new().merge(app).merge(forwarder_router);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`

    info!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let _ = axum::serve(listener, app.into_make_service()).await?;

    let _ = eventbus.stop().await?;
    Ok(())
}
