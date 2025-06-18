use regex::Regex;
use tracing::error;

#[inline]
pub fn bool_true() -> bool {
    true
}

#[inline]
pub fn bool_false() -> bool {
    false
}

pub fn serialize_re<S: serde::Serializer>(re: &Regex, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(re.as_str())
}

pub fn deserialize_re<'de, D: serde::Deserializer<'de>>(de: D) -> Result<Regex, D::Error> {
    struct Vis;
    impl<'de> serde::de::Visitor<'de> for Vis {
        type Value = Regex;

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Regex::new(v).unwrap_or_else(|e| {
                error!(error = %e, "Failed to parse regex - defaulting to empty regex");
                Regex::new("").unwrap()
            }))
        }

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a regex")
        }
    }

    de.deserialize_str(Vis)
}
