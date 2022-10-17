use nom::error::Error as NomError;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{alphanumeric1, char, multispace0, one_of},
    character::{is_alphabetic, is_digit},
    combinator::{map_res, opt, peek},
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
    fn boolean_operator(&self) -> fn(&str, &str) -> bool {
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
    pub key_value_pairs: Vec<(S, Selector, S)>,
}

/// Simple type alias to make returning this ridiculous thing easier.
type MatchSpecTuple<S> = (
    Option<S>,
    Option<S>,
    Option<S>,
    S,
    Option<S>,
    Option<S>,
    Option<Vec<(S, S, S)>>,
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

/// Parses the channel
/// ```
///  use matchspec::matchspec::channel_parser;
///
///  assert_eq!(channel_parser("conda-forge/linux-64::tensorflow"), Ok(("/linux-64::tensorflow", "conda-forge")));
///  assert_eq!(channel_parser("main::python"), Ok(("::python", "main")));
/// ```
pub fn channel_parser(s: &str) -> IResult<&str, &str> {
    terminated(take_while(is_alphanumeric_with_dashes), peek(one_of(":/")))(s)
}

/// Parses a single key_value_pair
/// ```
///  use matchspec::matchspec::key_value_pair_parser;
///
///  assert_eq!(key_value_pair_parser("subdir=linux-64"), Ok(("", ("subdir", "=", "linux-64"))));
/// ```
pub fn key_value_pair_parser(s: &str) -> IResult<&str, (&str, &str, &str)> {
    let value_parser = delimited(
        opt(one_of("'\"")),
        take_while(is_alphanumeric_with_dashes),
        opt(one_of("'\"")),
    );
    delimited(
        multispace0,
        tuple((alphanumeric1, selector_parser, value_parser)),
        multispace0,
    )(s)
}

/// Parses the whole matchspec using Nom, returing a `MatchSpecTuple`
/// Assumes this format:
/// `(channel(/subdir):(namespace):)name(version(build))[key1=value1,key2=value2]`
/// Instead of using this directly please use the `"".parse()` style provided by FromStr
fn parse_matchspec(s: &str) -> IResult<&str, MatchSpecTuple<&str>, NomError<&str>> {
    let subdir_parser = delimited(
        char('/'),
        take_while(is_alphanumeric_with_dashes),
        char(':'),
    );

    let namespace_parser = terminated(alphanumeric1, char(':'));
    let keys_vec_parser = delimited(
        char('['),
        many1(terminated(key_value_pair_parser, opt(char(',')))),
        char(']'),
    );

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

impl FromStr for MatchSpec<String> {
    type Err = NomError<String>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result = map_res(
            parse_matchspec,
            |(channel, subdir, namespace, package, s, v, keys)| {
                // Make sure an empty "" is, None, but convert to String otherwise.
                let version = match v {
                    Some("") => None,
                    Some(value) => Some(value),
                    _ => None,
                };
                // Convert inner into selector
                let selector: Option<Selector> = s.map(Selector::from);

                // Convert the key_value_pairs into (S, Selector, S) tuples.
                // I'm not sure its possible to have the full selector set, but this models it in a
                // good way.
                let key_value_pairs: Vec<(String, Selector, String)> = keys
                    .map(|vec: Vec<(&str, &str, &str)>| {
                        vec.iter()
                            .map(|(key, selector, value)| {
                                (
                                    key.to_string(),
                                    Selector::from(*selector),
                                    value.to_string(),
                                )
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                Ok::<MatchSpec<String>, Self::Err>(MatchSpec {
                    channel: channel.map(String::from),
                    subdir: subdir.map(String::from),
                    namespace: namespace.map(String::from),
                    package: package.into(),
                    selector,
                    version: version.map(String::from),
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
    mod component_parsers {
        use crate::matchspec::*;
        #[test]
        fn test_channel_parser() {
            assert_eq!(
                channel_parser("conda-forge::tensorflow >=2.9.1"),
                Ok(("::tensorflow >=2.9.1", "conda-forge"))
            );

            assert_eq!(
                channel_parser("main/linux-64::tensorflow >=2.9.1"),
                // Having this space here is ok because the selector_parser handles whitespace
                Ok(("/linux-64::tensorflow >=2.9.1", "main"))
            );
        }

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

        #[test]
        fn test_key_value_parser() {
            // Ensure we handle quoting
            assert_eq!(
                key_value_pair_parser("subdir = 'linux-64'"),
                Ok(("", ("subdir", "=", "linux-64"))),
            );

            assert_eq!(
                key_value_pair_parser("subdir = \"linux-64\""),
                Ok(("", ("subdir", "=", "linux-64"))),
            );

            // Also work without quoting
            assert_eq!(
                key_value_pair_parser("subdir = linux-64"),
                Ok(("", ("subdir", "=", "linux-64"))),
            );

            // Whitespace shouldn't matter
            assert_eq!(
                key_value_pair_parser("subdir=linux-64"),
                Ok(("", ("subdir", "=", "linux-64"))),
            );
        }
    }

    mod final_parser {
        use crate::matchspec::*;

        #[test]
        fn simple_package_and_version() {
            let result: Result<MatchSpec<String>, nom::error::Error<String>> =
                "tensorflow>=2.9.1".parse();

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
            let result: Result<MatchSpec<String>, nom::error::Error<String>> = "tensorflow".parse();

            let ms = result.unwrap();
            assert_eq!(ms.subdir, None);
            assert_eq!(ms.namespace, None);
            assert_eq!(ms.package, "tensorflow");
            assert_eq!(ms.version, None);
            assert!(ms.key_value_pairs.is_empty());
        }

        #[test]
        fn package_and_version_only() {
            let result: Result<MatchSpec<String>, nom::error::Error<String>> =
                "tensorflow>1".parse();

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
            let result: Result<MatchSpec<String>, nom::error::Error<String>> =
                "tensorflow>1[subdir!=win-64]".parse();

            let ms = result.unwrap();
            assert_eq!(ms.subdir, None);
            assert_eq!(ms.namespace, None);
            assert_eq!(ms.package, "tensorflow");
            assert_eq!(ms.version, Some("1".to_string()));
            assert_eq!(ms.selector, Some(Selector::GreaterThan));
            assert_eq!(ms.key_value_pairs.len(), 1);
            assert_eq!(
                ms.key_value_pairs.get(0),
                Some(&(
                    "subdir".to_string(),
                    Selector::NotEqualTo,
                    "win-64".to_string(),
                ))
            );
        }

        #[test]
        fn package_only_with_key_values() {
            let result: Result<MatchSpec<String>, nom::error::Error<String>> =
                "tensorflow[subdir=win-64]".parse();

            let ms = result.unwrap();
            assert_eq!(ms.subdir, None);
            assert_eq!(ms.namespace, None);
            assert_eq!(ms.package, "tensorflow");
            assert_eq!(ms.version, None);
            assert_eq!(ms.key_value_pairs.len(), 1);
            assert_eq!(
                ms.key_value_pairs.get(0),
                Some(&(
                    "subdir".to_string(),
                    Selector::EqualTo,
                    "win-64".to_string(),
                ))
            );
        }

        #[test]
        fn everything_specified() {
            let result: Result<MatchSpec<String>, nom::error::Error<String>> =
                "conda-forge/linux-64:UNUSED:tensorflow>2.9.1[license=GPL, subdir=linux-64]"
                    .parse();

            let ms = result.unwrap();
            assert_eq!(ms.channel, Some("conda-forge".to_string()));
            assert_eq!(ms.subdir, Some("linux-64".to_string()));
            assert_eq!(ms.namespace, Some("UNUSED".to_string()));
            assert_eq!(ms.package, "tensorflow");
            assert_eq!(ms.version, Some("2.9.1".to_string()));
            assert_eq!(ms.key_value_pairs.len(), 2);
            assert_eq!(
                ms.key_value_pairs.get(0),
                Some(&("license".to_string(), Selector::EqualTo, "GPL".to_string()))
            );

            assert_eq!(
                ms.key_value_pairs.get(1),
                Some(&(
                    "subdir".to_string(),
                    Selector::EqualTo,
                    "linux-64".to_string()
                ))
            );
        }
    }

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
    }

    mod real_life {
        use crate::matchspec::*;
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        #[test]
        fn repodata_depends() {
            let depends_file = format!(
                "{}/test_data/linux_64-depends.txt",
                env!("CARGO_MANIFEST_DIR")
            );
            let repodata_depends_buffer =
                BufReader::new(File::open(depends_file).expect("opening repodata depends file"));
            let depends: Vec<String> = repodata_depends_buffer
                .lines()
                .map(|l| l.unwrap())
                .collect();

            for line in depends {
                let _parsed: MatchSpec<String> = line
                    .parse()
                    .unwrap_or_else(|_| panic!("Failed to parse: {}", line));
            }
        }
    }
}
