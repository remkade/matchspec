use crate::matchspec::*;
use crate::input_table::*;
use nom::error::{Error as NomError, ErrorKind};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{alphanumeric0, alphanumeric1, satisfy, multispace0, multispace1, one_of},
    combinator::{complete, eof, opt, peek},
    multi::separated_list0,
    sequence::{delimited, terminated, tuple},
    IResult,
};
use version_compare::{Version};

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
/// *Note:* Compatible Release is not yet supported and will be mapped to `Selector::EqualTo`
pub(crate) fn selector_parser(s: &str) -> IResult<&str, &str> {
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
pub(crate) fn name_parser(s: &str) -> IResult<&str, &str> {
    take_while1(is_any_valid_str_with_glob)(s)
}

/// Parses the package version
pub(crate) fn version_parser(s: &str) -> IResult<&str, &str> {
    let (remainder, version) = take_while1(is_any_valid_str_with_glob)(s)?;
    match Version::from(version) {
        Some(_) => { Ok((remainder, version)) }
        None => {
            Err(nom::Err::Failure(NomError {
                code: ErrorKind::Fail,
                input: "Version parse failed",
            }))
        }
    }
}

fn version_and_selector_parser(s: &str) -> IResult<&str, (&str, &str)> {
    tuple((selector_parser, version_parser))(s)
}

pub(crate) fn compound_selector_parser(s: &str) -> IResult<&str, CompoundSelector<String>> {
    let result = tuple((
        version_and_selector_parser,
        delimited(multispace0, satisfy(is_comma_or_alt), multispace0),
        version_and_selector_parser,
    ))(s);

    // If we can parse via the more exhaustive parser, return that.
    match result {
        Ok((remainder, parsed)) => Ok((remainder, parsed.into())),
        Err(_) => {
            match version_and_selector_parser(s) {
                Ok((remainder, parsed)) => Ok((remainder, parsed.into())),
                Err(err) => Err(err),
            }
        }
    }
}

/// Parses the channel
pub(crate) fn channel_parser(s: &str) -> IResult<&str, &str> {
    terminated(take_while(is_alphanumeric_with_dashes), peek(one_of(":/")))(s)
}

/// Parses a single key_value_pair:
/// `key=value`
pub(crate) fn key_value_pair_parser(s: &str) -> IResult<&str, (&str, &str, &str)> {
    let value_parser = delimited(
        opt(satisfy(is_quote)),
        take_while1(is_alphanumeric_with_dashes),
        opt(complete(satisfy(is_quote))),
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
pub(crate) fn implicit_matchspec_parser(s: &str) -> IResult<&str, MatchSpec<String>> {
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
pub(crate) fn full_matchspec_parser(s: &str) -> IResult<&str, MatchSpec<String>, NomError<&str>> {
    // Eats `/subdir`
    let subdir_parser = delimited(
        satisfy(is_forward_slash),
        take_while(is_alphanumeric_with_dashes),
        peek(satisfy(is_colon)),
    );

    // Eats `:namespace:`
    let namespace_parser = delimited(satisfy(is_colon), alphanumeric0, satisfy(is_colon));

    // Eats `[ .. ]`
    let keys_vec_parser = delimited(
        satisfy(is_left_bracket),
        separated_list0(satisfy(is_comma), key_value_pair_parser),
        satisfy(is_right_bracket),
    );


    // Put all the parsers together
    let (remainder, t) = complete(tuple((
        opt(channel_parser),
        opt(subdir_parser),
        opt(namespace_parser),
        name_parser,
        opt(compound_selector_parser),
        opt(keys_vec_parser),
    )))(s)?;

    Ok((remainder, t.into()))
}

#[cfg(test)]
mod test {
    mod component_parsers {
        use crate::parsers::*;
        use nom::error::{ErrorKind};

        #[test]
        fn test_channel_parser() {
            assert_eq!(
                channel_parser("conda-forge::tensorflow >=2.9.1"),
                Ok(("::tensorflow >=2.9.1", "conda-forge"))
            );

            assert_eq!(
                channel_parser("main/linux-64::tensorflow >=2.9.1"),
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
            assert_eq!(version_parser("not-correct-version"),
                       Err(nom::Err::Failure(NomError {
                           code: ErrorKind::Fail,
                           input: "Version parse failed",
                       })));
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
                (
                    "tensorflow",
                    Some(CompoundSelector::Single {
                        selector: Selector::EqualTo,
                        version: "2.9.1".to_string(),
                    }),
                    None
                )
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
                    Some(CompoundSelector::Single {
                        selector: Selector::EqualTo,
                        version: "2.9.1".to_string(),
                    }),
                    Some("mkl_py39hb9fcb14_0".to_string())
                ),
            );

            // Verify that we don't match an explicit matchspec
            let explicit = implicit_matchspec_parser("tensorflow > 2.9.1");
            assert_eq!(
                explicit,
                Err(nom::Err::Error(NomError {
                    code: ErrorKind::Eof,
                    input: " > 2.9.1",
                }))
            );
        }
    }

    mod final_parser {
        use crate::error::MatchSpecError;
        use crate::matchspec::*;

        #[test]
        fn simple_package_and_version() {
            let base: MatchSpec<String> = MatchSpec::default();
            let expected = MatchSpec {
                package: "tensorflow".to_string(),
                version: Some(CompoundSelector::Single {
                    selector: Selector::GreaterThanOrEqualTo,
                    version: "2.9.1".to_string(),
                }),
                ..base
            };

            let ms: MatchSpec<String> = "tensorflow>=2.9.1".parse().unwrap();

            assert_eq!(ms, expected);
        }

        #[test]
        fn package_only() {
            let result: Result<MatchSpec<String>, MatchSpecError> = "tensorflow".parse();

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
                version: Some(CompoundSelector::Single {
                    selector: Selector::EqualTo,
                    version: "2.9.1".to_string(),
                }),
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
            let expected = MatchSpec {
                package: "tensorflow".to_string(),
                version: Some(CompoundSelector::Single {
                    selector: Selector::EqualTo,
                    version: "2.9.1".to_string(),
                }),
                key_value_pairs: Vec::new(),
                build: Some("mkl_py39hb9fcb14_0".to_string()),
                channel: None,
                subdir: None,
                namespace: None,
            };

            // Test the explicit matcher first
            let explicit: MatchSpec<String> = "tensorflow==2.9.1[build=\"mkl_py39hb9fcb14_0\"]"
                .parse()
                .unwrap();

            // Test against expected
            assert_eq!(expected, explicit);

            // Test the implicit matcher second
            let implicit: MatchSpec<String> =
                "tensorflow 2.9.1 mkl_py39hb9fcb14_0".parse().unwrap();

            // Test against expected
            assert_eq!(expected, explicit);

            // They should both be equal
            assert_eq!(implicit, explicit);
        }

        #[test]
        fn package_and_version_with_key_values() {
            let result: Result<MatchSpec<String>, MatchSpecError> =
                "tensorflow>1[subdir!=win-64]".parse();

            let ms = result.unwrap();
            assert_eq!(ms.subdir, None);
            assert_eq!(ms.namespace, None);
            assert_eq!(ms.package, "tensorflow");
            assert_eq!(
                ms.version,
                Some(CompoundSelector::Single {
                    selector: Selector::GreaterThan,
                    version: "1".to_string(),
                })
            );
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
            let result: Result<MatchSpec<String>, MatchSpecError> =
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
        fn everything_except_namespace() {
            let ms: MatchSpec<String> = "main/linux-64::pytorch>1.10.2".parse().unwrap();

            let expected = MatchSpec {
                channel: Some("main".to_string()),
                subdir: Some("linux-64".to_string()),
                namespace: None,
                package: "pytorch".to_string(),
                build: None,
                version: Some(CompoundSelector::Single {
                    selector: Selector::GreaterThan,
                    version: "1.10.2".to_string(),
                }),
                key_value_pairs: Vec::new(),
            };

            assert_eq!(ms, expected);
        }

        #[test]
        fn everything_specified() {
            let expected = MatchSpec {
                channel: Some("conda-forge".to_string()),
                subdir: Some("linux-64".to_string()),
                namespace: Some("UNUSED".to_string()),
                package: "tensorflow".to_string(),
                build: None,
                version: Some(CompoundSelector::And {
                    first_selector: Selector::GreaterThan,
                    first_version: "2.9.1".to_string(),
                    second_selector: Selector::LessThan,
                    second_version: "3.0.0".to_string(),
                }),
                key_value_pairs: Vec::new(),
            };

            let ms: MatchSpec<String> =
                "conda-forge/linux-64:UNUSED:tensorflow>2.9.1,<3.0.0[license=GPL, subdir=linux-64]"
                    .parse()
                    .unwrap();

            assert_eq!(expected, ms);
        }

        #[test]
        fn fail_on_wrong_semver_version() {
            let ms: Result<MatchSpec<String>, MatchSpecError> = "python=wrong".parse();
            assert_eq!(ms, Err(MatchSpecError { message: "Version parse failed".to_string() }))
        }
    }

    // This is a suite of tests using real data from things like the repodata.json
    #[cfg(test)]
    mod real_life {
        use crate::matchspec::MatchSpec;
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
