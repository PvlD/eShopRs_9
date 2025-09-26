use url::Url;

struct VersionInt {
    major_version: usize,
    minor_version: Option<usize>,
}

const API_VERSION_PARAMETER: &str = "api-version";

pub struct QueryStringApiVersion {
    #[allow(dead_code)]
    version: VersionInt,
    str_value: String,
}

impl QueryStringApiVersion {
    pub fn append_to_url(&self, url: &mut Url) {
        url.query_pairs_mut().append_pair(API_VERSION_PARAMETER, &self.str_value);
    }

    fn new(v: VersionInt) -> Self {
        Self {
            str_value: QueryStringApiVersion::string_val(&v),
            version: v,
        }
    }

    fn string_val(version_int: &VersionInt) -> String {
        if let Some(minor_version) = version_int.minor_version {
            return format!("{}.{}", version_int.major_version, minor_version);
        }

        format!("{}", version_int.major_version)
    }
}

impl From<usize> for QueryStringApiVersion {
    fn from(major_version: usize) -> Self {
        let v = VersionInt { major_version, minor_version: None };
        QueryStringApiVersion::new(v)
    }
}

impl From<(usize, usize)> for QueryStringApiVersion {
    fn from(t: (usize, usize)) -> Self {
        let (major_version, minor_version) = t;
        let v = VersionInt {
            major_version,
            minor_version: Some(minor_version),
        };
        QueryStringApiVersion::new(v)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_version() {
        let mut url = Url::parse("https://api.spotify.com/v1/search").unwrap();
        let v1 = QueryStringApiVersion::from(1);
        v1.append_to_url(&mut url);
        assert_eq!(url.as_str(), "https://api.spotify.com/v1/search?api-version=1");

        let mut url = Url::parse("https://api.spotify.com/v1/search").unwrap();
        let v12 = QueryStringApiVersion::from((1, 2));
        v12.append_to_url(&mut url);
        assert_eq!(url.as_str(), "https://api.spotify.com/v1/search?api-version=1.2");
    }
}
