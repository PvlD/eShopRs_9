use std::{collections::HashMap, env};

use log::warn;
use regex::RegexBuilder;

type Data = HashMap<String, String>;

pub struct InternalImpl {
    url_map: Data,
}

impl InternalImpl {
    pub fn new_from_env() -> Self {
        let f = || {
            let mut data = Data::new();

            let pat = r"^services__(.*)__(https|http)__(\d+)$";
            let re = RegexBuilder::new(pat).case_insensitive(false).build().unwrap();

            for (key, value) in env::vars() {
                //print!("{}: {}\n", key, value);
                if re.is_match(&key)
                    && let Some((_, [api, protocol, _])) = re.captures(&key).map(|caps| caps.extract())
                {
                    let api = api.to_string();
                    let api = if api.contains("_") {
                        // if env read from file where '-' forbidden
                        warn!("replace '_' with '-' in {api}");
                        api.replace("_", "-")
                    } else {
                        api
                    };

                    let service_key = format!("{protocol}://{api}");

                    if let std::collections::hash_map::Entry::Vacant(e) = data.entry(service_key) {
                        e.insert(value);
                    } else {
                        warn!("  ignored   {key} : {value}  ");
                    }
                }
            }

            data
        };
        Self { url_map: f() }
    }

    pub fn new_from_vec(vec: &Vec<(&str, &str)>) -> Self {
        let f = || {
            let mut data = Data::new();

            vec.iter().for_each(|(key, value)| {
                if data.contains_key(*key) {
                    warn!("  ignored   {key} : {value}  ");
                } else {
                    data.insert(key.to_string(), value.to_string());
                }
            });
            data
        };
        Self { url_map: f() }
    }

    fn get_mapped(&self, key: &str) -> Option<&str> {
        self.url_map.get(key).map(|s| s.as_str())
    }
}

impl super::UrlMap for InternalImpl {
    fn get_mapped_url(&self, key: &str) -> Option<&str> {
        self.get_mapped(key)
    }

    fn to_string(&self) -> String {
        format!("{:?}", self.url_map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn create_map_service_from_env() {
        let td = vec![
            ("services__apiservice__http__0", "http://localhost:5455", "http://apiservice"),
            ("services__apiservice__https__0", "https://localhost:7356", "https://apiservice"),
            ("services__apiservice__https__1", "https://localhost:7356", "https://apiservice"),
        ];

        td.iter().for_each(|(k, v, _)| unsafe { env::set_var(k, v) });

        let s = InternalImpl::new_from_env();

        assert_eq!(td[0].1, s.get_mapped(td[0].2).unwrap());
        assert_eq!(td[1].1, s.get_mapped(td[1].2).unwrap());
    }
    #[test]
    pub fn create_map_service_from_vec() {
        let td = vec![("http://apiservice", "http://localhost:5455"), ("https://apiservice", "https://localhost:7356"), ("https://apiservice", "https://localhost:7356")];

        let s = InternalImpl::new_from_vec(&td);

        assert_eq!(td[0].1, s.get_mapped(td[0].0).unwrap());
        assert_eq!(td[1].1, s.get_mapped(td[1].0).unwrap());
        assert_eq!(td[2].1, s.get_mapped(td[2].0).unwrap());
    }
}
