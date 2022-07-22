use std::{borrow::Cow, fmt, fmt::Debug};

#[derive(Debug)]
pub struct FormatDescription<T: Debug> {
    source: Cow<'static, str>,
    parts: Vec<FormatPart<T>>,
}

#[derive(Debug, thiserror::Error)]
pub enum FormatParseError<E>
where
    E: Debug + fmt::Display,
{
    #[error("Cannot interpolate: {0}")]
    InterpolationError(E),
    #[error("Expected closing brace at column {0}")]
    ExpectedClosingBrace(usize),
}

pub trait InterpolationProvider: Sized {
    type Source;
    type ParseError;
    type FormatError;

    fn parse_provider(name: &str) -> Result<Self, Self::ParseError>;
    fn format<W: fmt::Write>(
        &self,
        source: &Self::Source,
        f: &mut W,
    ) -> Result<(), Self::FormatError>;
}

#[derive(Debug, thiserror::Error)]
pub enum FormatError<T: InterpolationProvider>
where
    T::FormatError: Debug + fmt::Display,
{
    #[error("Format error: {0}")]
    Std(#[from] fmt::Error),
    #[error("Interpolation error: {0}")]
    Interpolation(T::FormatError),
}

#[derive(Debug)]
enum FormatPart<T: Debug> {
    Char(char),
    SourceString { start: usize, end: usize },
    Interpolate(T),
}

enum ParseState {
    Empty,
    Source { start: usize },
    Interpolate { start: usize },
    OpenBrace,
}

impl<T> TryFrom<Cow<'static, str>> for FormatDescription<T>
where
    T: InterpolationProvider + Debug,
    T::ParseError: Debug + fmt::Display,
{
    type Error = FormatParseError<T::ParseError>;

    fn try_from(value: Cow<'static, str>) -> Result<Self, Self::Error> {
        let mut parts = Vec::new();
        let mut state = ParseState::Empty;
        for (idx, c) in value.char_indices() {
            state = match state {
                ParseState::Empty if c == '{' => ParseState::OpenBrace,
                ParseState::Empty => ParseState::Source { start: idx },
                ParseState::Source { start } if c == '{' => {
                    parts.push(FormatPart::SourceString { start, end: idx });
                    ParseState::OpenBrace
                }
                ParseState::Source { start } => ParseState::Source { start },
                ParseState::Interpolate { start } if c == '}' => {
                    let interpolation = &value[start..idx];
                    let interpolation = T::parse_provider(interpolation)
                        .map_err(FormatParseError::InterpolationError)?;
                    parts.push(FormatPart::Interpolate(interpolation));
                    ParseState::Empty
                }
                ParseState::Interpolate { start } => ParseState::Interpolate { start },
                ParseState::OpenBrace if c == '{' => {
                    parts.push(FormatPart::Char('{'));
                    ParseState::Empty
                }
                ParseState::OpenBrace => ParseState::Interpolate { start: idx },
            };
        }
        match state {
            ParseState::Source { start } => {
                parts.push(FormatPart::SourceString {
                    start,
                    end: value.len(),
                });
            }
            ParseState::Empty => {}
            _ => return Err(Self::Error::ExpectedClosingBrace(value.len())),
        };
        Ok(Self {
            parts,
            source: value,
        })
    }
}

impl<T> FormatDescription<T>
where
    T: InterpolationProvider + Debug,
    T::FormatError: Debug + fmt::Display,
{
    pub fn format<W: fmt::Write>(
        &self,
        source: &T::Source,
        f: &mut W,
    ) -> Result<(), FormatError<T>> {
        for part in &self.parts {
            match part {
                FormatPart::Char(c) => f.write_char(*c)?,
                FormatPart::SourceString { start, end } => {
                    f.write_str(&self.source.as_ref()[*start..*end])?;
                }
                FormatPart::Interpolate(int) => {
                    int.format(source, f).map_err(FormatError::Interpolation)?;
                }
            }
        }
        Ok(())
    }

    pub fn format_to_string(&self, source: &T::Source) -> Result<String, FormatError<T>> {
        let mut string = String::new();
        self.format(source, &mut string)?;
        Ok(string)
    }
}

impl<T: Debug> FormatDescription<T> {
    pub fn raw(source: impl Into<Cow<'static, str>>) -> Self {
        let source = source.into();
        Self {
            parts: vec![FormatPart::SourceString {
                start: 0,
                end: source.len(),
            }],
            source,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fmt;

    struct Source(usize, usize);
    #[derive(Debug)]
    enum Inter {
        UseA,
        UseB,
        Both,
    }
    impl InterpolationProvider for Inter {
        type Source = Source;
        type ParseError = anyhow::Error;
        type FormatError = fmt::Error;

        fn parse_provider(name: &str) -> Result<Self, Self::ParseError> {
            match name {
                "a" => Ok(Self::UseA),
                "b" => Ok(Self::UseB),
                "both" => Ok(Self::Both),
                _ => Err(anyhow::anyhow!("No interpolation")),
            }
        }

        fn format<W: fmt::Write>(
            &self,
            source: &Self::Source,
            f: &mut W,
        ) -> Result<(), Self::FormatError> {
            match self {
                Inter::UseA => write!(f, "{}", source.0),
                Inter::UseB => write!(f, "{}", source.1),
                Inter::Both => write!(f, "{}{}", source.0, source.1),
            }
        }
    }

    #[test]
    fn basic() {
        for (lookup, source, expected) in [
            ("{a} = {b}".to_owned(), Source(2, 3), "2 = 3"),
            ("nothing".to_owned(), Source(2, 3), "nothing"),
            ("{a} = 2".to_owned(), Source(2, 3), "2 = 2"),
            ("b = {b}".to_owned(), Source(2, 3), "b = 3"),
            ("1 = {both}{a}{b}".to_owned(), Source(2, 3), "1 = 2323"),
        ] {
            let descr = FormatDescription::<Inter>::try_from(Cow::from(lookup)).unwrap();
            assert_eq!(&descr.format_to_string(&source).unwrap(), expected);
        }
    }

    #[test]
    fn parse_errors() {
        assert!(matches!(
            FormatDescription::<Inter>::try_from(Cow::from("{a} b {a")),
            Err(FormatParseError::ExpectedClosingBrace(8))
        ));
        assert!(matches!(
            FormatDescription::<Inter>::try_from(Cow::from("{c}")),
            Err(FormatParseError::InterpolationError(_))
        ));
    }
}
