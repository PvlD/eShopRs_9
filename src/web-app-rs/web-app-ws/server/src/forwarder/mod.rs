pub mod product_images;

type Client = reqwest::Client;

use anyhow::{Context, Result};
use axum::{body::HttpBody, extract::Request};
use axum::{
    body::{Body, Bytes},
    response::Response,
};

//use log::info;
use tokio_stream::{StreamExt, wrappers::UnboundedReceiverStream};

async fn forward(url: &str, req: Request, client: Client) -> Result<Response> {
    //info!("Sent request to {url} ");

    let method = req.method().clone();

    let mut body_stream = req.into_body().into_data_stream();

    let resp = if body_stream.is_end_stream() {
        let reqwest_response = client.request(method, url);

        let resp = reqwest_response.send().await;

        resp.with_context(|| format!("Error sending request to {url}"))?
    } else {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let body_copy_h = tokio::spawn(async move {
            loop {
                let r: Result<Option<Bytes>, axum::Error> = body_stream.try_next().await;

                match r {
                    Ok(Some(b)) => match tx.send(Ok(b)) {
                        Ok(_) => {}
                        Err(e) => {
                            return Err(e);
                        }
                    },
                    Ok(None) => {
                        return Ok(());
                    }
                    Err(e) => match tx.send(Err(e)) {
                        Ok(_) => {
                            return Ok(());
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    },
                }
            }
        });

        let reqwest_response = client.request(method, url).body(reqwest::Body::wrap_stream(UnboundedReceiverStream::new(rx)));

        let resp = reqwest_response.send().await;

        let body_copy_err = body_copy_h.await.unwrap();

        let _ = body_copy_err.with_context(|| "Error copying body")?;

        resp.with_context(|| "Error sending request")?
    };

    let response_builder = Response::builder().status(resp.status().as_u16());

    response_builder.body(Body::from_stream(resp.bytes_stream())).with_context(|| "Error creating response")
}
