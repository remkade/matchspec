use crate::error::MatchSpecError;
use crate::input_table::*;
use crate::package_candidate::*;
use crate::parsers::*;
use nom::branch::alt;
use nom::error::Error as NomError;
use nom::Finish;
use pyo3::prelude::*;
use std::fmt::Debug;
use std::str::FromStr;
use version_compare::{compare_to, Cmp};

/// Matches a string with a string (possibly) containing globs
fn is_match_glob_str(glob_str: &str, match_str: &str) -> bool {
    let mut index: Option<usize> = Some(0);
    let mut it = glob_str.split('*').peekable();
    while let Some(part) = it.next() {
        index = match_str.get(index.unwrap()..).and_then(|s| s.find(part));
        if index.is_none() || (it.peek().is_none() && !match_str.ends_with(part)) {
            return false;
        }
    }
    true
}

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
            Selector::EqualTo => Selector::eq,
            Selector::NotEqualTo => Selector::ne,
            Selector::LessThan => Selector::lt,
            Selector::LessThanOrEqualTo => Selector::le,
            Selector::GreaterThan => Selector::gt,
            Selector::GreaterThanOrEqualTo => Selector::ge,
        }
    }
    fn eq(a: &str, b: &str) -> bool {
        compare_to(a, b, Cmp::Eq).unwrap_or(false)
    }

    fn ne(a: &str, b: &str) -> bool {
        compare_to(a, b, Cmp::Ne).unwrap_or(false)
    }
    fn lt(a: &str, b: &str) -> bool {
        compare_to(a, b, Cmp::Lt).unwrap_or(false)
    }
    fn le(a: &str, b: &str) -> bool {
        compare_to(a, b, Cmp::Le).unwrap_or(false)
    }
    fn gt(a: &str, b: &str) -> bool {
        compare_to(a, b, Cmp::Gt).unwrap_or(false)
    }
    fn ge(a: &str, b: &str) -> bool {
        compare_to(a, b, Cmp::Ge).unwrap_or(false)
    }
}

/// CompoundSelector is a grouping of selector and version pairs. For example, in these MatchSpecs:
/// ```text
///  gcc>9|!=10.0.1 # GCC must be greater than 9.* OR not 10.0.1
///  python>=3.0.0,<3.7.2 # Python must be greater than or equal to 3.0.0 AND less than 3.7.2
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
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

impl Default for CompoundSelector<String> {
    fn default() -> Self {
        CompoundSelector::Single {
            selector: Selector::GreaterThanOrEqualTo,
            version: "0".to_string(),
        }
    }
}

/// Create a selector from a parser tuple:
/// ```
/// use rust_matchspec::{Selector, CompoundSelector};
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

impl<S, V> From<((S, V), char, (S, V))> for CompoundSelector<String>
    where
        S: Into<Selector>,
        V: Into<String>,
{
    fn from((one, boolean, two): ((S, V), char, (S, V))) -> Self {
        match boolean {
            '|' => CompoundSelector::Or { first_selector: one.0.into(), first_version: one.1.into(), second_selector: two.0.into(), second_version: two.1.into() },
            ',' => CompoundSelector::And { first_selector: one.0.into(), first_version: one.1.into(), second_selector: two.0.into(), second_version: two.1.into() },
            _ => panic!("You must use either | or , as the separator when converting into a CompoundSelector"),
        }
    }
}

impl<S> CompoundSelector<S>
    where
        S: AsRef<str> + PartialEq + Into<String>,
{
    /// This takes a versions and tests that it falls within the constraints of this CompoundSelector
    /// ```
    ///  use rust_matchspec::{Selector, CompoundSelector};
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
    pub fn is_match(&self, other: &str) -> bool {
        match self {
            CompoundSelector::Single { selector, version } => {
                selector.boolean_operator()(other, version.as_ref())
            }
            CompoundSelector::And {
                first_selector,
                first_version,
                second_selector,
                second_version,
            } => {
                first_selector.boolean_operator()(other, first_version.as_ref())
                    && second_selector.boolean_operator()(other, second_version.as_ref())
            }
            CompoundSelector::Or {
                first_selector,
                first_version,
                second_selector,
                second_version,
            } => {
                first_selector.boolean_operator()(other, first_version.as_ref())
                    || second_selector.boolean_operator()(other, second_version.as_ref())
            }
        }
    }
}

/// Create a selector from a parser tuple:
/// ```
/// use rust_matchspec::{Selector, CompoundSelector};
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
#[pyclass]
#[derive(Debug, Clone, Eq)]
pub struct MatchSpec {
    pub channel: Option<String>,
    pub subdir: Option<String>,
    pub namespace: Option<String>,
    pub package: String,
    pub version: Option<CompoundSelector<String>>,
    pub build: Option<String>,
    pub build_number: Option<CompoundSelector<String>>,
    pub key_value_pairs: Vec<(String, CompoundSelector<String>)>,
}

/// Custom implementation to make sure that we don't compare key_value_pairs
/// If we don't know how to understand it, we should ignore the key value for the purpose of struct
/// equality. Makes it simpler to handle potentially unknown future additions to the spec.
impl PartialEq for MatchSpec {
    fn eq(&self, other: &Self) -> bool {
        self.channel == other.channel
            && self.subdir == other.subdir
            && self.namespace == other.namespace
            && self.package == other.package
            && self.version == other.version
            && self.build == other.build
    }
}

impl Default for MatchSpec {
    fn default() -> Self {
        MatchSpec {
            channel: None,
            subdir: None,
            namespace: None,
            package: "*".to_string(),
            version: None,
            build: None,
            build_number: None,
            key_value_pairs: Vec::new(),
        }
    }
}

/// This is where we actually do the parsing
impl FromStr for MatchSpec {
    type Err = MatchSpecError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match alt((implicit_matchspec_parser, full_matchspec_parser))(s).finish() {
            Ok((_, ms)) => Ok(ms),
            Err(NomError { input, code: _ }) => Err(MatchSpecError {
                message: String::from(input),
            }),
        }
    }
}

impl From<(&str, Option<&str>, Option<&str>)> for MatchSpec {
    fn from((package, version, build): (&str, Option<&str>, Option<&str>)) -> Self {
        MatchSpec {
            channel: None,
            subdir: None,
            namespace: None,
            package: package.into(),
            version: version.map(|s| CompoundSelector::Single {
                selector: Selector::EqualTo,
                version: s.into(),
            }),
            build: build.map(|s| s.into()),
            build_number: None,
            key_value_pairs: Vec::new(),
        }
    }
}

impl
From<(
    Option<&str>,
    Option<&str>,
    Option<&str>,
    &str,
    Option<CompoundSelector<String>>,
    Option<Vec<(&str, CompoundSelector<String>)>>,
)> for MatchSpec
{
    fn from(
        (channel, subdir, ns, package, cs, keys): (
            Option<&str>,
            Option<&str>,
            Option<&str>,
            &str,
            Option<CompoundSelector<String>>,
            Option<Vec<(&str, CompoundSelector<String>)>>,
        ),
    ) -> Self {
        let namespace = if let Some(a) = ns {
            if a.is_empty() {
                None
            } else {
                Some(a.to_string())
            }
        } else {
            None
        };

        // Create the initial struct based on the parsed tuple
        let mut ms = MatchSpec {
            channel: channel.map(|s| s.into()),
            subdir: subdir.map(|s| s.into()),
            namespace,
            package: package.into(),
            version: cs,
            build: None,
            build_number: None,
            key_value_pairs: Vec::new(),
        };

        // Convert the key_value_pairs into (S, Selector, S) tuples.
        // I'm not sure its possible to have the full selector set, but this models it in a
        // pretty good way.
        let key_value_pairs: Vec<(String, CompoundSelector<String>)> = keys.map(|vec| {
            vec.iter().map(|(key, value)| {
                (String::from(*key), value.clone())
            }).collect()
        }).unwrap_or_default();

        // Lets set the final attributes based on the key value pairs
        // Currently we only support EqualTo relations, but maybe in the future we can fix that.
        for (key, compound_selector) in &key_value_pairs {
            // TODO: here
            match (key.as_ref(), compound_selector) {
                ("build", CompoundSelector::Single { selector: Selector::EqualTo, version }) => ms.build = Some(version.clone()),
                ("channel", CompoundSelector::Single { selector: Selector::EqualTo, version }) => ms.channel = Some(version.clone()),
                ("subdir", CompoundSelector::Single { selector: Selector::EqualTo, version }) => ms.subdir = Some(version.clone()),
                ("namepsace", CompoundSelector::Single { selector: Selector::EqualTo, version }) => ms.namespace = Some(version.clone()),
                ("build_number", CompoundSelector::Single { selector: _, version: _ }) => ms.build_number = Some(compound_selector.clone()),
                _ => (),
            }
        }

        // Save all the key value pairs, this is done last to avoid borrow after move
        ms.key_value_pairs = key_value_pairs;
        ms
    }
}

impl MatchSpec {
    /// Matches package names. The matchspec package may contain globs
    /// ```
    /// use rust_matchspec::matchspec::*;
    ///
    /// let ms: MatchSpec = "openssl>1.1.1a".parse().unwrap();
    /// assert!(ms.is_package_match("openssl".to_string()));
    /// ```
    pub fn is_package_match(&self, package: String) -> bool {
        package.chars().all(is_alphanumeric_with_dashes)
            && is_match_glob_str(self.package.as_ref(), package.as_ref())
    }

    /// Uses the Selector embedded in the matchspec to do a match on only a version
    /// ```
    /// use rust_matchspec::matchspec::*;
    ///
    /// let ms: MatchSpec = "openssl>1.1.1a".parse().unwrap();
    /// assert!(ms.is_version_match(&"1.1.1r"));
    /// ```
    pub fn is_version_match(&self, version: &str) -> bool {
        self.version
            .as_ref()
            .map(|v| v.is_match(version))
            .unwrap_or(true)
    }

    pub fn is_package_version_match(&self, package: &str, version: &str) -> bool {
        package.chars().all(is_alphanumeric_with_dashes)
            && is_match_glob_str(self.package.as_ref(), package)
            && self.is_version_match(version)
    }
}

impl MatchSpec {
    pub fn is_match(&self, pc: &PackageCandidate) -> bool {
        let is_equal = |a: &Option<String>, b: &Option<String>| a.is_none() || a == b;

        self.is_package_version_match(&pc.name, pc.version.as_ref().unwrap_or(&String::new()))
            && self.is_build_number_match(&pc.build_number)
            && is_equal(&self.subdir, &pc.subdir)
            && is_equal(&self.build, &pc.build)
    }

    pub fn is_build_number_match(&self, build_number: &Option<u32>) -> bool {
        match build_number {
            Some(number) => {
                self.build_number
                    .as_ref()
                    .map(|v| v.is_match(&number.to_string()))
                    .unwrap_or(true)
            }
            None => true
        }
    }
}

#[cfg(test)]
mod test {
    #[cfg(test)]
    mod matching {
        use crate::matchspec::*;

        #[test]
        fn package_only() {
            let mut ms: MatchSpec = "tensorflow".parse().unwrap();

            assert!(ms.is_package_match("tensorflow".to_string()));
            assert!(!ms.is_package_match("pytorch".to_string()));

            ms = "tensor*-gpu".parse().unwrap();
            assert!(ms.is_package_match("tensorflow-gpu".to_string()));
            assert!(!ms.is_package_match("tennnnsorflow-gpu".to_string()));

            ms = "tensorflow*".parse().unwrap();
            assert!(ms.is_package_match("tensorflow".to_string()));
            assert!(ms.is_package_match("tensorflow-gpu".to_string()));

            ms = "*-gpu".parse().unwrap();
            assert!(ms.is_package_match("tensorflow-gpu".to_string()));
            assert!(!ms.is_package_match("tensorflow".to_string()));

            // Illegal chars
            assert!(!ms.is_package_match("python>3.10[name=* vmd5=\"abcdef1312\"]".to_string()));
        }

        #[test]
        fn package_and_version_only() {
            let ms: MatchSpec = "tensorflow>1.9.2".parse().unwrap();

            assert!(ms.is_package_version_match("tensorflow", "1.9.3"));
            assert!(!ms.is_package_version_match("tensorflow", "1.9.0"));
        }

        #[test]
        fn compound_selectors() {
            let single = CompoundSelector::Single {
                selector: Selector::GreaterThan,
                version: "1.1.1",
            };

            assert!(single.is_match("1.2.1"));
            assert!(single.is_match("3.0.0"));
            assert!(!single.is_match("1.1.1"));
            assert!(!single.is_match("0.1.1"));

            let and = CompoundSelector::And {
                first_selector: Selector::GreaterThan,
                first_version: "1.1.1",
                second_selector: Selector::LessThanOrEqualTo,
                second_version: "1.2.1",
            };

            assert!(and.is_match("1.2.1"));
            assert!(and.is_match("1.1.7"));
            assert!(!and.is_match("1.2.2"));
            assert!(!and.is_match("0.1.1"));

            let or = CompoundSelector::Or {
                first_selector: Selector::LessThan,
                first_version: "1.1.1",
                second_selector: Selector::GreaterThan,
                second_version: "1.2.1",
            };

            assert!(or.is_match("3.0.0"));
            assert!(or.is_match("0.1.1"));
            assert!(!or.is_match("1.2.1"));
            assert!(!or.is_match("1.1.1"));
            assert!(!or.is_match("1.1.7"));
        }

        #[test]
        fn test_version_compare() {
            let ms: MatchSpec = "python>3.6".parse().unwrap();
            assert!(!ms.is_package_version_match("python", "3.5"));
            assert!(ms.is_package_version_match("python", "3.8"));
            assert!(ms.is_package_version_match("python", "3.9"));
            assert!(ms.is_package_version_match("python", "3.10"));
        }
    }
}
