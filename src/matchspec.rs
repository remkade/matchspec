use nom::error::Error as NomError;
use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_while},
    character::complete::{alphanumeric1, char, multispace0, one_of},
    character::{is_alphabetic, is_digit},
    combinator::{map_res, opt},
    multi::many1,
    sequence::{delimited, terminated, tuple},
    Finish, IResult,
};
use std::fmt::Debug;
use std::str::FromStr;

/// Enum that is used to group
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Selector {
    GreaterThan,
    GreaterThanOrEqualTo,
    LessThan,
    LessThanOrEqualTo,
    NotEqualTo,
    EqualTo,
}

impl From<&str> for Selector {
    fn from(value: &str) -> Self {
        match value {
            ">" => Self::GreaterThan,
            ">=" => Self::GreaterThanOrEqualTo,
            "<" => Self::LessThan,
            "<=" => Self::LessThanOrEqualTo,
            "!=" => Self::NotEqualTo,
            _ => Self::EqualTo,
        }
    }
}

/// Tests for alphanumeric with dashes, underscores, or periods
/// ```
/// use matchspec::matchspec::is_alphanumeric_with_dashes;
///
/// assert!("_123abc".chars().all(is_alphanumeric_with_dashes));
/// ```
pub fn is_alphanumeric_with_dashes(c: char) -> bool {
    is_alphabetic(c as u8) || is_digit(c as u8) || c == '-' || c == '_'
}

/// Tests for alphanumeric with dashes, underscores, or periods
/// ```
/// use matchspec::matchspec::is_alphanumeric_with_dashes_or_period;
///
///  assert!("_.123abc".chars().all(is_alphanumeric_with_dashes_or_period));
///  assert!("conda-forge".chars().all(is_alphanumeric_with_dashes_or_period));
///  assert!("1.1.1a".chars().all(is_alphanumeric_with_dashes_or_period));
///  assert_eq!(false, "!#$&*#(".chars().all(is_alphanumeric_with_dashes_or_period));
/// ```
pub fn is_alphanumeric_with_dashes_or_period(c: char) -> bool {
    is_alphanumeric_with_dashes(c) || c == '.'
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
#[derive(Debug, Clone, Default)]
pub struct MatchSpec {
    pub channel: Option<String>,
    pub subdir: Option<String>,
    pub namespace: Option<String>,
    pub package: String,
    pub selector: Option<Selector>,
    pub version: Option<String>,
    pub key_value_pairs: Vec<(String, Selector, String)>,
}

impl MatchSpec {
    /// Create a MatchSpec using only package and selector
    fn simple(package: &str, selector: Selector, version: &str) -> Self {
        MatchSpec {
            channel: None,
            subdir: None,
            namespace: None,
            package: String::from(package),
            selector: Some(selector),
            version: Some(String::from(version)),
            key_value_pairs: vec![],
        }
    }
}

/// Simple type alias to make returning this ridiculous thing easier.
type MatchSpecTuple<'a> = (
    Option<&'a str>,
    Option<&'a str>,
    Option<&'a str>,
    &'a str,
    Option<&'a str>,
    Option<&'a str>,
    Option<Vec<(&'a str, &'a str, &'a str)>>,
);

/// Parses a version selector. Possible values:
/// | Selector | Function                                                                   |
/// |----------|----------------------------------------------------------------------------|
/// | >        | Greater Than                                                               |
/// | <        | Less Than                                                                  |
/// | >=       | Greater Than or Equal To                                                   |
/// | <=       | Less Than or Equal To                                                      |
/// | ==       | Equal                                                                      |
/// | =        | Equal                                                                      |
/// | !=       | Not Equal                                                                  |
/// | ~=       | [Compatible Release](https://peps.python.org/pep-0440/#compatible-release) |
///
/// ```
///  use matchspec::matchspec::selector_parser;
///
///  assert_eq!(selector_parser("!=2.9.1"), Ok(("2.9.1", "!=")));
///  assert_eq!(selector_parser(">2.9.1"), Ok(("2.9.1", ">")));
/// ```
pub fn selector_parser(s: &str) -> IResult<&str, &str> {
    delimited(
        multispace0,
        alt((
            tag("==="),
            tag("!="),
            tag(">="),
            tag("<="),
            tag("=="),
            tag("~="),
            tag("="),
            tag(">"),
            tag("<"),
        )),
        multispace0,
    )(s)
}

/// Parses the package name
/// ```
///  use matchspec::matchspec::name_parser;
///
///  assert_eq!(name_parser("tensorflow >=2.9.1"), Ok((" >=2.9.1", "tensorflow")));
///  assert_eq!(name_parser("tensorflow>=2.9.1"), Ok((">=2.9.1", "tensorflow")));
/// ```
pub fn name_parser(s: &str) -> IResult<&str, &str> {
    take_while(is_alphanumeric_with_dashes_or_period)(s)
}

/// Parses the package version
/// ```
///  use matchspec::matchspec::version_parser;
///
///  assert_eq!(version_parser("2.9.1"), Ok(("", "2.9.1")));
///  assert_eq!(version_parser("2.9.1[subdir=linux]"), Ok(("[subdir=linux]", "2.9.1")));
/// ```
pub fn version_parser(s: &str) -> IResult<&str, &str> {
    take_while(is_alphanumeric_with_dashes_or_period)(s)
}

/// Parses the whole matchspec using Nom, returing a `MatchSpecTuple`
/// Assumes this format:
/// `(channel(/subdir):(namespace):)name(version(build))[key1=value1,key2=value2]`
fn parse_matchspec(s: &str) -> IResult<&str, MatchSpecTuple, NomError<&str>> {
    let channel_parser = terminated(take_while(is_alphanumeric_with_dashes), one_of(":/"));
    let subdir_parser = delimited(
        char(':'),
        take_while(is_alphanumeric_with_dashes),
        char('/'),
    );
    let namespace_parser = delimited(char(':'), alphanumeric1, char(':'));
    let key_value_pair_parser = tuple((alphanumeric1, selector_parser, is_not("],")));
    let keys_vec_parser = delimited(char('['), many1(key_value_pair_parser), char(']'));

    tuple((
        opt(channel_parser),
        opt(subdir_parser),
        opt(namespace_parser),
        name_parser,
        opt(selector_parser),
        opt(version_parser),
        opt(keys_vec_parser),
    ))(s)
}

impl FromStr for MatchSpec {
    type Err = NomError<String>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result: Result<(&str, MatchSpec), NomError<&str>> = map_res(
            parse_matchspec,
            |(channel, subdir, namespace, package, s, v, keys)| {
                // Make sure an empty "" is, None, but convert to String otherwise.
                let version: Option<String> = match v {
                    Some("") => None,
                    Some(value) => Some(value.to_string()),
                    _ => None
                };
                let selector: Option<Selector> = s.map(Selector::from);

                // Convert the key_value_pairs into (String, Selector, String) tuples.
                // I'm not sure its possible to have the full selector set, but this models it in a
                // good way.
                let key_value_pairs: Vec<(String, Selector, String)> = keys
                    .map(|vec: Vec<(&str, &str, &str)>| {
                        vec.iter()
                            .map(|(key, selector, value)| {
                                (
                                    String::from(*key),
                                    Selector::from(*selector),
                                    String::from(*value),
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                Ok::<MatchSpec, Self::Err>(MatchSpec {
                    channel: channel.map(String::from),
                    subdir: subdir.map(String::from),
                    namespace: namespace.map(String::from),
                    package: package.into(),
                    selector,
                    version,
                    key_value_pairs,
                })
            },
        )(s)
        .finish();

        match result {
            Ok((_, ms)) => Ok(ms),
            Err(NomError { input, code }) => Err(NomError {
                input: String::from(input),
                code,
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_spec_creation() {
        let spec = MatchSpec::simple("tensorflow", Selector::GreaterThan, "2.9.1");
        assert_eq!(spec.package, String::from("tensorflow"));
        assert!(spec.namespace.is_none());
        assert!(spec.key_value_pairs.is_empty());
    }

    mod component_parsers {
        use crate::matchspec::*;

        #[test]
        fn test_name_parser() {
            assert_eq!(
                name_parser("tensorflow >=2.9.1"),
                // Having this space here is ok because the selector_parser handles whitespace
                Ok((" >=2.9.1", "tensorflow"))
            );
            assert_eq!(
                name_parser("tensorflow>=2.9.1"),
                Ok((">=2.9.1", "tensorflow"))
            );
            assert_eq!(name_parser("openssl>=1.1.1a"), Ok((">=1.1.1a", "openssl")));
            assert_eq!(
                name_parser("vs2017_win-64==19.16.27032.1"),
                Ok(("==19.16.27032.1", "vs2017_win-64"))
            );
        }

        #[test]
        fn test_selector_parser() {
            assert_eq!(selector_parser(" >=2.9.1"), Ok(("2.9.1", ">=")));
            assert_eq!(selector_parser("!= 2.9.1"), Ok(("2.9.1", "!=")));
            assert_eq!(selector_parser(">=1.1.1a"), Ok(("1.1.1a", ">=")));
            assert_eq!(
                selector_parser("==19.16.27032.1"),
                Ok(("19.16.27032.1", "=="))
            );
            assert_eq!(
                selector_parser(" ~= 19.16.27032.1"),
                Ok(("19.16.27032.1", "~="))
            );
            assert_eq!(
                selector_parser(" === 19.16.27032.1"),
                Ok(("19.16.27032.1", "==="))
            );
        }

        #[test]
        fn test_version_parser() {
            assert_eq!(version_parser("19.16.27032.1"), Ok(("", "19.16.27032.1")));
            assert_eq!(version_parser("2.9.1"), Ok(("", "2.9.1")));
            assert_eq!(version_parser("4.3.post1"), Ok(("", "4.3.post1")));
            assert_eq!(version_parser("5.0.0.1"), Ok(("", "5.0.0.1")));
            assert_eq!(version_parser("2022.1"), Ok(("", "2022.1")));
            assert_eq!(version_parser("1.21_5"), Ok(("", "1.21_5")));
            assert_eq!(
                version_parser("2.9.1[subdir=linux]"),
                Ok(("[subdir=linux]", "2.9.1"))
            );
        }
    }

    mod final_parser {
        use crate::matchspec::*;

        #[test]
        fn simple_package_and_version() {
            let result: Result<MatchSpec, nom::error::Error<String>> = "tensorflow>=2.9.1".parse();

            if result.is_err() {
                assert_eq!("", format!("Nom Error: {:?}", result));
            }

            let ms = result.unwrap();
            assert_eq!(ms.package, "tensorflow");
            assert_eq!(ms.version, Some("2.9.1".to_string()));
            assert_eq!(ms.selector, Some(Selector::GreaterThanOrEqualTo));
        }

        #[test]
        fn package_only() {
            let result: Result<MatchSpec, nom::error::Error<String>> = "tensorflow".parse();

            let ms = result.unwrap();
            assert_eq!(ms.subdir, None);
            assert_eq!(ms.namespace, None);
            assert_eq!(ms.package, "tensorflow");
            assert_eq!(ms.version, None);
            assert!(ms.key_value_pairs.is_empty());
        }

        #[test]
        fn package_and_version_only() {
            let result: Result<MatchSpec, nom::error::Error<String>> = "tensorflow>1".parse();

            let ms = result.unwrap();
            assert_eq!(ms.subdir, None);
            assert_eq!(ms.namespace, None);
            assert_eq!(ms.package, "tensorflow");
            assert_eq!(ms.version, Some("1".to_string()));
            assert_eq!(ms.selector, Some(Selector::GreaterThan));
            assert!(ms.key_value_pairs.is_empty());
        }

        #[test]
        fn package_and_version_with_key_values() {
            let result: Result<MatchSpec, nom::error::Error<String>> =
                "tensorflow>1[subdir!=win-64]".parse();

            let ms = result.unwrap();
            assert_eq!(ms.subdir, None);
            assert_eq!(ms.namespace, None);
            assert_eq!(ms.package, "tensorflow");
            assert_eq!(ms.version, Some("1".to_string()));
            assert_eq!(ms.selector, Some(Selector::GreaterThan));
            assert!(ms.key_value_pairs.len() == 1);
            assert_eq!(
                ms.key_value_pairs.get(0),
                Some(&(
                    "subdir".to_string(),
                    Selector::NotEqualTo,
                    "win-64".to_string()
                ))
            );
        }

        #[test]
        fn package_only_with_key_values() {
            let result: Result<MatchSpec, nom::error::Error<String>> =
                "tensorflow[subdir=win-64]".parse();

            let ms = result.unwrap();
            assert_eq!(ms.subdir, None);
            assert_eq!(ms.namespace, None);
            assert_eq!(ms.package, "tensorflow");
            assert_eq!(ms.version, None);
            assert!(ms.key_value_pairs.len() == 1);
            assert_eq!(
                ms.key_value_pairs.get(0),
                Some(&(
                    "subdir".to_string(),
                    Selector::EqualTo,
                    "win-64".to_string()
                ))
            );
        }

        #[test]
        fn everything_specified() {
            let result: Result<MatchSpec, nom::error::Error<String>> =
                "conda-forge/linux-64:UNUSED:tensorflow>2.9.1[license=GPL]".parse();

            let ms = result.unwrap();
            assert_eq!(ms.channel, Some("conda-forge".to_string()));
            assert_eq!(ms.subdir, Some("linux-64".to_string()));
            assert_eq!(ms.namespace, Some("UNUSED".to_string()));
            assert_eq!(ms.package, "tensorflow".to_string());
            assert_eq!(ms.version, None);
            assert!(ms.key_value_pairs.len() == 1);
            assert_eq!(
                ms.key_value_pairs.get(0),
                Some(&(
                    "subdir".to_string(),
                    Selector::EqualTo,
                    "win-64".to_string()
                ))
            );
        }
    }
}
