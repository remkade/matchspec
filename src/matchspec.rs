use crate::parsers::*;
use nom::branch::alt;
use nom::error::Error as NomError;
use nom::Finish;
use std::fmt::Debug;
use std::str::FromStr;

/// Enum that is used for representating the selector types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selector {
    GreaterThan,
    GreaterThanOrEqualTo,
    LessThan,
    LessThanOrEqualTo,
    NotEqualTo,
    EqualTo,
}

impl<S> From<S> for Selector
where
    S: AsRef<str>,
{
    fn from(value: S) -> Self {
        match value.as_ref() {
            ">" => Self::GreaterThan,
            ">=" => Self::GreaterThanOrEqualTo,
            "<" => Self::LessThan,
            "<=" => Self::LessThanOrEqualTo,
            "!=" => Self::NotEqualTo,
            _ => Self::EqualTo,
        }
    }
}

impl Selector {
    pub fn boolean_operator(&self) -> fn(&str, &str) -> bool {
        match self {
            Selector::EqualTo => str::eq,
            Selector::NotEqualTo => str::ne,
            Selector::LessThan => str::lt,
            Selector::LessThanOrEqualTo => str::le,
            Selector::GreaterThan => str::gt,
            Selector::GreaterThanOrEqualTo => str::ge,
        }
    }
}

/// CompoundSelector is a grouping of selector and version pairs. For example, in these MatchSpecs:
/// ```text
///  gcc>9|!=10.0.1 # GCC must be greater than 9.* OR not 10.0.1
///  python>=3.0.0,<3.7.2 # Python must be greater than or equal to 3.0.0 AND less than 3.7.2
/// ```
#[derive(Debug, PartialEq, Eq)]
pub enum CompoundSelector<S>
where
    S: Into<String> + AsRef<str>,
{
    Single {
        selector: Selector,
        version: S,
    },
    And {
        first_selector: Selector,
        first_version: S,
        second_selector: Selector,
        second_version: S,
    },
    Or {
        first_selector: Selector,
        first_version: S,
        second_selector: Selector,
        second_version: S,
    },
}

/// Create a selector from a parser tuple:
/// ```
/// use matchspec::{Selector, CompoundSelector};
///
/// let cs = CompoundSelector::from((">", "1.1.1"));
/// assert_eq!(cs, CompoundSelector::Single{
///     selector: Selector::GreaterThan,
///     version: "1.1.1".to_string(),
/// });
/// ```
impl<S, V> From<(S, V)> for CompoundSelector<String>
where
    S: Into<Selector>,
    V: Into<String>,
{
    fn from(input: (S, V)) -> Self {
        CompoundSelector::Single {
            selector: input.0.into(),
            version: input.1.into(),
        }
    }
}

impl<S> CompoundSelector<S>
where
    S: AsRef<str> + PartialEq + Into<String>,
{
    /// This takes a versions and tests that it falls within the constraints of this CompoundSelector
    /// ```
    ///  use matchspec::{Selector, CompoundSelector};
    ///
    ///  let single = CompoundSelector::Single {
    ///     selector: Selector::GreaterThan,
    ///     version: "1.1.1",
    ///  };
    ///  
    ///  assert!(single.is_match(&"1.2.1"));
    ///  assert!(single.is_match(&"3.0.0"));
    ///  assert!(!single.is_match(&"1.1.1"));
    ///  assert!(!single.is_match(&"0.1.1"));
    ///  
    ///  let and = CompoundSelector::And {
    ///     first_selector: Selector::GreaterThan,
    ///     first_version: "1.1.1",
    ///     second_selector: Selector::LessThanOrEqualTo,
    ///     second_version: "1.2.1",
    ///  };
    ///  
    ///  assert!(and.is_match(&"1.2.1"));
    ///  assert!(and.is_match(&"1.1.7"));
    ///  assert!(!and.is_match(&"1.2.2"));
    ///  assert!(!and.is_match(&"0.1.1"));
    ///  
    ///  let or = CompoundSelector::Or {
    ///     first_selector: Selector::LessThan,
    ///     first_version: "1.1.1",
    ///     second_selector: Selector::GreaterThan,
    ///     second_version: "1.2.1",
    ///  };
    ///  
    ///  assert!(or.is_match(&"3.0.0"));
    ///  assert!(or.is_match(&"0.1.1"));
    ///  assert!(!or.is_match(&"1.2.1"));
    ///  assert!(!or.is_match(&"1.1.1"));
    ///  assert!(!or.is_match(&"1.1.7"));
    ///  ```
    pub fn is_match(&self, other: &S) -> bool {
        match self {
            CompoundSelector::Single { selector, version } => {
                selector.boolean_operator()(other.as_ref(), version.as_ref())
            }
            CompoundSelector::And {
                first_selector,
                first_version,
                second_selector,
                second_version,
            } => {
                first_selector.boolean_operator()(other.as_ref(), first_version.as_ref())
                    && second_selector.boolean_operator()(other.as_ref(), second_version.as_ref())
            }
            CompoundSelector::Or {
                first_selector,
                first_version,
                second_selector,
                second_version,
            } => {
                first_selector.boolean_operator()(other.as_ref(), first_version.as_ref())
                    || second_selector.boolean_operator()(other.as_ref(), second_version.as_ref())
            }
        }
    }
}

/// Create a selector from a parser tuple:
/// ```
/// use matchspec::{Selector, CompoundSelector};
///
/// let cs = CompoundSelector::from((">", "1.1.1", ",", "<", "3.0.0"));
/// assert_eq!(cs, CompoundSelector::And{
///     first_selector: Selector::GreaterThan,
///     first_version: "1.1.1".to_string(),
///     second_selector: Selector::LessThan,
///     second_version: "3.0.0".to_string(),
/// });
/// ```
impl<S, V> From<(S, V, V, S, V)> for CompoundSelector<String>
where
    S: Into<Selector>,
    V: Into<String> + AsRef<str> + PartialEq + std::fmt::Display,
{
    fn from(
        (first_selector, first_version, joiner, second_selector, second_version): (S, V, V, S, V),
    ) -> Self {
        match joiner.as_ref() {
            "|" => CompoundSelector::Or {
                first_selector: first_selector.into(),
                first_version: first_version.into(),
                second_selector: second_selector.into(),
                second_version: second_version.into(),
            },
            "," => CompoundSelector::And {
                first_selector: first_selector.into(),
                first_version: first_version.into(),
                second_selector: second_selector.into(),
                second_version: second_version.into(),
            },
            // Should be impossible to hit this if you are instantiating this from a parser
            _ => panic!(
                "Unable to create CompoundSelector, invalid joiner '{}'",
                joiner
            ),
        }
    }
}

/// This struct encodes the conda MatchSpec language as a datatype.
/// ## Examples
/// Conda recognizes this as the official MatchSpec Structure
/// `(channel(/subdir):(namespace):)name(version(build))[key1=value1,key2=value2]`
///
/// Here are some examples in real usage.
/// ```bash
/// openssl>=1.1.1g
/// openssl>=1.1.1g[platform='linux-64']
/// tensorflow==2.9.*
/// ```
/// Full MatchSpec documentation is found in the code [here](https://github.com/conda/conda/blob/main/conda/models/match_spec.py)
/// and [here](https://conda.io/projects/conda-build/en/latest/resources/package-spec.html#build-version-spec) in the spec
#[derive(Debug, Clone, Default, Eq)]
pub struct MatchSpec<S>
where
    S: AsRef<str> + PartialEq + PartialOrd,
{
    pub channel: Option<S>,
    pub subdir: Option<S>,
    pub namespace: Option<S>,
    pub package: S,
    pub selector: Option<Selector>,
    pub version: Option<S>,
    pub build: Option<S>,
    pub key_value_pairs: Vec<(S, Selector, S)>,
}

/// Custom implementation to make sure that we don't compare key_value_pairs
/// If we don't know how to understand it, we should ignore the key value for the purpose of struct
/// equality. Makes it simpler to handle potentially unknown future additions to the spec.
impl<S> PartialEq for MatchSpec<S>
where
    S: AsRef<str> + PartialEq + PartialOrd,
{
    fn eq(&self, other: &Self) -> bool {
        self.channel == other.channel
            && self.subdir == other.subdir
            && self.namespace == other.namespace
            && self.package == other.package
            && self.selector == other.selector
            && self.version == other.version
            && self.build == other.build
    }
}

impl FromStr for MatchSpec<String> {
    type Err = NomError<String>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match alt((implicit_matchspec_parser, full_matchspec_parser))(s).finish() {
            Ok((_, ms)) => Ok(ms),
            Err(NomError { input, code }) => Err(NomError {
                input: String::from(input),
                code,
            }),
        }
    }
}

impl<S> From<(S, Option<S>, Option<S>)> for MatchSpec<String>
where
    S: AsRef<str>,
{
    fn from((package, version, build): (S, Option<S>, Option<S>)) -> Self {
        let selector: Option<Selector> = version.as_ref().map(|_| Selector::EqualTo);
        MatchSpec {
            channel: None,
            subdir: None,
            namespace: None,
            package: package.as_ref().to_string(),
            selector,
            version: version.map(|s| s.as_ref().to_string()),
            build: build.map(|s| s.as_ref().to_string()),
            key_value_pairs: Vec::new(),
        }
    }
}

impl<S>
    From<(
        Option<S>,
        Option<S>,
        Option<S>,
        S,
        Option<S>,
        Option<S>,
        Option<Vec<(S, S, S)>>,
    )> for MatchSpec<String>
where
    S: Into<String> + AsRef<str> + PartialEq,
{
    fn from(
        (channel, subdir, namespace, package, s, version, keys): (
            Option<S>,
            Option<S>,
            Option<S>,
            S,
            Option<S>,
            Option<S>,
            Option<Vec<(S, S, S)>>,
        ),
    ) -> Self {
        // Convert inner into selector
        let selector: Option<Selector> = s.map(Selector::from);

        // Convert the key_value_pairs into (S, Selector, S) tuples.
        // I'm not sure its possible to have the full selector set, but this models it in a
        // pretty good way.
        let key_value_pairs: Vec<(String, Selector, String)> = keys
            .map(|vec: Vec<(S, S, S)>| {
                vec.iter()
                    .map(|(key, selector, value)| {
                        (
                            key.as_ref().to_string(),
                            Selector::from(selector),
                            value.as_ref().to_string(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Create the initial struct based on the parsed tuple
        let mut ms = MatchSpec {
            channel: channel.map(|s| s.into()),
            subdir: subdir.map(|s| s.into()),
            namespace: namespace.map(|s| s.into()),
            package: package.into(),
            selector,
            version: version.map(|s| s.into()),
            build: None,
            key_value_pairs: Vec::new(),
        };

        // Lets set the final attributes based on the key value pairs
        // Currently we only support EqualTo relations, but maybe in the future we can fix that.
        for (key, selector, value) in &key_value_pairs {
            match (key.as_ref(), selector, value) {
                ("build", Selector::EqualTo, _) => ms.build = Some(value.clone()),
                ("channel", Selector::EqualTo, _) => ms.channel = Some(value.clone()),
                ("subdir", Selector::EqualTo, _) => ms.subdir = Some(value.clone()),
                ("namepsace", Selector::EqualTo, _) => ms.namespace = Some(value.clone()),
                _ => (),
            }
        }

        // Save all the key value pairs, this is done last to avoid borrow after move
        ms.key_value_pairs = key_value_pairs;
        ms
    }
}

impl<S: AsRef<str> + PartialOrd + PartialEq<str>> MatchSpec<S> {
    /// Does simple &str equality matching against the package name
    /// ```
    /// use ::matchspec::*;
    ///
    /// let ms: MatchSpec<String> = "openssl>1.1.1a".parse().unwrap();
    /// assert!(ms.is_package_match("openssl".to_string()));
    /// ```
    pub fn is_package_match(&self, package: S) -> bool {
        self.package == package
    }

    /// Uses the Selector embedded in the matchspec to do a match on only a version
    /// ```
    /// use ::matchspec::*;
    ///
    /// let ms: MatchSpec<String> = "openssl>1.1.1a".parse().unwrap();
    /// assert!(ms.is_version_match("1.1.1r".to_string()));
    /// ```
    pub fn is_version_match(&self, version: S) -> bool {
        self.selector
            .as_ref()
            .zip(self.version.as_ref())
            .map(|(s, v)| s.boolean_operator()(version.as_ref(), v.as_ref()))
            .unwrap_or(false)
    }

    pub fn is_package_version_match(&self, package: S, version: S) -> bool {
        self.package == package && self.is_version_match(version)
    }
}

#[cfg(test)]
mod test {
    #[cfg(test)]
    mod matching {
        use crate::matchspec::*;

        #[test]
        fn package_only() {
            let ms: MatchSpec<String> = "tensorflow".parse().unwrap();

            assert!(ms.is_package_match("tensorflow".to_string()));
            assert!(!ms.is_package_match("pytorch".to_string()));
        }

        #[test]
        fn package_and_version_only() {
            let ms: MatchSpec<String> = "tensorflow>1.9.2".parse().unwrap();

            assert!(ms.is_package_version_match("tensorflow".to_string(), "1.9.3".to_string()));
            assert!(!ms.is_package_version_match("tensorflow".to_string(), "1.9.0".to_string()));
        }

        #[test]
        fn compound_selectors() {
            let single = CompoundSelector::Single {
                selector: Selector::GreaterThan,
                version: "1.1.1",
            };

            assert!(single.is_match(&"1.2.1"));
            assert!(single.is_match(&"3.0.0"));
            assert!(!single.is_match(&"1.1.1"));
            assert!(!single.is_match(&"0.1.1"));

            let and = CompoundSelector::And {
                first_selector: Selector::GreaterThan,
                first_version: "1.1.1",
                second_selector: Selector::LessThanOrEqualTo,
                second_version: "1.2.1",
            };

            assert!(and.is_match(&"1.2.1"));
            assert!(and.is_match(&"1.1.7"));
            assert!(!and.is_match(&"1.2.2"));
            assert!(!and.is_match(&"0.1.1"));

            let or = CompoundSelector::Or {
                first_selector: Selector::LessThan,
                first_version: "1.1.1",
                second_selector: Selector::GreaterThan,
                second_version: "1.2.1",
            };

            assert!(or.is_match(&"3.0.0"));
            assert!(or.is_match(&"0.1.1"));
            assert!(!or.is_match(&"1.2.1"));
            assert!(!or.is_match(&"1.1.1"));
            assert!(!or.is_match(&"1.1.7"));
        }
    }
}
