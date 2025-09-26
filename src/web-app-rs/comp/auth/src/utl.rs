use url::{Position, Url};

fn url_from_str(s: &str) -> Result<(Url, bool), url::ParseError> {
    let u1 = Url::parse(s);

    match u1 {
        Ok(url) => Ok((url, false)),
        Err(url::ParseError::RelativeUrlWithoutBase) => {
            let fake_base = "http://example.com";
            let fake_url = Url::parse(fake_base).unwrap();

            match fake_url.join(s) {
                Ok(url) => Ok((url, true)),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

pub(crate) fn urlstring_from_str(s: &str) -> Result<String, url::ParseError> {
    match url_from_str(s) {
        Ok((url, true)) => {
            let url = &url[Position::BeforePath..];
            Ok(url.to_string())
        }
        Ok((url, false)) => Ok(url.to_string()),
        Err(e) => Err(e),
    }
}

#[allow(dead_code)]
pub(crate) fn url_string_relative_from_str(s: &str) -> Result<String, url::ParseError> {
    match url_from_str(s) {
        Ok((url, _)) => {
            let url = &url[Position::BeforePath..];
            Ok(url.to_string())
        }
        Err(e) => Err(e),
    }
}
