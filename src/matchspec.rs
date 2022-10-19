use nom::error::Error as NomError;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{alphanumeric1, char, multispace0, multispace1, one_of},
    character::{is_alphabetic, is_digit},
    combinator::{complete, eof, opt, peek},
    multi::separated_list0,
    sequence::{delimited, terminated, tuple},
    Finish, IResult,
};
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
    take_while1(is_alphanumeric_with_dashes_or_period)(s)
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
        take_while1(is_alphanumeric_with_dashes),
        opt(one_of("'\"")),
    );

    delimited(
        multispace0,
        tuple((alphanumeric1, selector_parser, value_parser)),
        multispace0,
    )(s)
}

/// Implicit MatchSpec Parser for the simple space separated form.
/// Example formats:
///
/// ```bash
/// zstd 1.4.5 h9ceee32_0
/// python 2.7.*
/// _libgcc_mutex 0.1 main
/// backports_abc 0.5 py27h7b3c97b_0
/// ```
fn implicit_matchspec_parser(s: &str) -> IResult<&str, MatchSpec<String>> {
    let (remainder, t) = tuple((
        take_while1(is_alphanumeric_with_dashes_or_period),
        opt(delimited(multispace1, version_parser, multispace0)),
        opt(take_while1(is_alphanumeric_with_dashes_or_period)),
        eof,
    ))(s)?;

    let output = (t.0, t.1, t.2);
    Ok((remainder, output.into()))
}

/// Parses the whole matchspec using Nom, returning a `MatchSpecTuple`
/// Assumes this format:
/// `(channel(/subdir):(namespace):)name(version(build))[key1=value1,key2=value2]`
/// Instead of using this directly please use the `"".parse()` style provided by FromStr
fn full_matchspec_parser(s: &str) -> IResult<&str, MatchSpec<String>, NomError<&str>> {
    let subdir_parser = delimited(
        char('/'),
        take_while(is_alphanumeric_with_dashes),
        char(':'),
    );

    let namespace_parser = terminated(alphanumeric1, char(':'));
    let keys_vec_parser = delimited(
        char('['),
        separated_list0(char(','), key_value_pair_parser),
        char(']'),
    );

    let (remainder, t) = complete(tuple((
        opt(channel_parser),
        opt(subdir_parser),
        opt(namespace_parser),
        name_parser,
        opt(selector_parser),
        opt(version_parser),
        opt(keys_vec_parser),
    )))(s)?;

    Ok((remainder, t.into()))
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

        #[test]
        fn test_implicit_parser() {
            // Package only
            let (_, package_only) = implicit_matchspec_parser("tensorflow").unwrap();
            assert_eq!(
                (
                    package_only.package.as_ref(),
                    package_only.version,
                    package_only.build
                ),
                ("tensorflow", None, None)
            );

            let (_, package_version) = implicit_matchspec_parser("tensorflow 2.9.1").unwrap();
            assert_eq!(
                (
                    package_version.package.as_ref(),
                    package_version.version,
                    package_version.build
                ),
                ("tensorflow", Some("2.9.1".to_string()), None)
            );

            let (_, everything) =
                implicit_matchspec_parser("tensorflow 2.9.1 mkl_py39hb9fcb14_0").unwrap();
            assert_eq!(
                (
                    everything.package.as_ref(),
                    everything.version,
                    everything.build
                ),
                (
                    "tensorflow",
                    Some("2.9.1".to_string()),
                    Some("mkl_py39hb9fcb14_0".to_string())
                ),
            );

            // Verify that we don't match an explicit matchspec
            let explicit = implicit_matchspec_parser("tensorflow > 2.9.1");
            assert_eq!(
                explicit,
                Err(nom::Err::Error(NomError {
                    code: nom::error::ErrorKind::Eof,
                    input: " > 2.9.1"
                }))
            );
        }
    }

    mod final_parser {
        use crate::matchspec::*;

        #[test]
        fn simple_package_and_version() {
            let base: MatchSpec<String> = MatchSpec::default();
            let expected = MatchSpec {
                package: "tensorflow".to_string(),
                selector: Some(Selector::GreaterThanOrEqualTo),
                version: Some("2.9.1".to_string()),
                ..base
            };

            let ms: MatchSpec<String> = "tensorflow>=2.9.1".parse().unwrap();

            assert_eq!(ms, expected);
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

        /// Matchspecs can effectively have 2 valid representations of version and packagename
        /// matchers. The most explicit form is: `tensorflow==2.9.1`, but the other supported mode
        /// is the implicit: `tensorflow 2.9.1`. Both are supported, and they are equivalent.
        #[test]
        fn package_and_version_only() {
            let base: MatchSpec<String> = MatchSpec::default();

            // Our output should look like this
            let expected = MatchSpec {
                package: "tensorflow".to_string(),
                selector: Some(Selector::EqualTo),
                version: Some("2.9.1".to_string()),
                ..base
            };

            // Test the explicit matcher first
            let explicit: MatchSpec<String> = "tensorflow==2.9.1".parse().unwrap();
            assert_eq!(explicit, expected);

            // Test the implicit matcher second
            let implicit: MatchSpec<String> = "tensorflow 2.9.1".parse().unwrap();
            assert_eq!(implicit, expected);

            // They should both be equal
            assert_eq!(implicit, explicit);
        }

        /// Another common matchspec is `package version build`. Like so:
        /// `tensorflow 2.9.1 mkl_py39hb9fcb14_0`
        /// This is equivalent to:
        /// `tensorflow==2.9.1[build="mkl_py39hb9fcb14_0"]`
        #[test]
        fn package_version_build_implicit_matcher() {
            // Test the explicit matcher first
            let explicit: MatchSpec<String> = "tensorflow==2.9.1[build=\"mkl_py39hb9fcb14_0\"]"
                .parse()
                .unwrap();

            assert_eq!(explicit.subdir, None);
            assert_eq!(explicit.namespace, None);
            assert_eq!(explicit.package, "tensorflow");
            assert_eq!(explicit.version, Some("2.9.1".to_string()));
            assert_eq!(explicit.selector, Some(Selector::EqualTo));
            assert!(!explicit.key_value_pairs.is_empty());
            assert_eq!(explicit.build, Some("mkl_py39hb9fcb14_0".to_string()));

            // Test the implicit matcher second
            let implicit: MatchSpec<String> =
                "tensorflow 2.9.1 mkl_py39hb9fcb14_0".parse().unwrap();
            assert_eq!(implicit.subdir, None);
            assert_eq!(implicit.namespace, None);
            assert_eq!(implicit.package, "tensorflow");
            assert_eq!(implicit.version, Some("2.9.1".to_string()));
            assert_eq!(implicit.selector, Some(Selector::EqualTo));
            assert_eq!(explicit.build, Some("mkl_py39hb9fcb14_0".to_string()));
            assert!(implicit.key_value_pairs.is_empty());

            // They should both be equal
            assert_eq!(implicit, explicit);
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
            assert_eq!(ms.subdir, Some("win-64".to_string()));
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

    // This is a suite of tests using real data from things like the repodata.json
    #[cfg(test)]
    mod real_life {
        use crate::matchspec::*;
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        /// This is a test that loads data from repodata.json
        /// Here's how that file was generated:
        ///
        /// ```bash
        /// curl https://repo.anaconda.com/pkgs/main/linux-64/repodata.json | jq -rM '.packages | map(.depends) | flatten | .[]' > test_data/linux_64-depends.txt
        /// ```
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
