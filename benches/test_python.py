from conda.models.match_spec import MatchSpec
from pathlib import Path
import rust_matchspec
import json

test_data = Path('./test_data')
depends_file = test_data / 'linux_64-depends.txt'
repodata_file = test_data / 'repodata-linux-64.json'


def bench_match_against_matchspec(list: [str]):
    """ Takes the list of matchspecs and matches against python 3.9.1 """
    for item in list:
        rust_matchspec.match_against_matchspec(item, 'python', '3.9.1')


def test_rust_matchspec_on_repodata_depends(benchmark):
    """
    Test the rust_matchspec.match_against_matchspec using the
    linux_64-depends.txt file.
    """
    with open(depends_file) as f:
        depends = f.readlines()

    benchmark(bench_match_against_matchspec, list=depends)


def bench_conda_against_repodata_depends(list: [str]):
    """
    Runs a list of matchspecs against a static package, this is a little
    contrived, but it is meant to compare the instantiation and filtering speed
    of MatchSpec
    """
    for item in list:
        ms = MatchSpec(item)
        ms.match({'name': 'python', 'version': '3.9.1',
                 'build': 'hbdb9e5c_0', 'build_number': 0})


def test_conda_matchspec_on_repodata_depends(benchmark):
    """
    Test Conda's MatchSpec against the linux_64-depends.txt file
    """
    with open(depends_file) as f:
        depends = f.readlines()

    benchmark(bench_conda_against_repodata_depends, list=depends)


def bench_rust_matchspec_filter_package_list(list: [dict[str, str]]):
    """
    Runs rust_matchspec.filter_package_list() against a list of packages
    """
    _matches = rust_matchspec.filter_package_list('python>=3.9.1', list)


def test_rust_matchspec_filter_package_list(benchmark):
    """
    Test rust_matchspec's filter_package_list() against the full linux-64
    repodata.json from Anaconda's defaults.
    """
    with open(repodata_file) as f:
        repodata = list(json.load(f)['packages'].values())

    benchmark(bench_rust_matchspec_filter_package_list, list=repodata)


def bench_conda_filter_package_list(list: [dict[str, str]]):
    """
    Runs uses MatchSpec against a list of packages to filter out non-matches
    """
    ms = MatchSpec('python>=3.9.1')
    _matches = [p for p in list if ms.match(p)]


def test_conda_filter_package_list(benchmark):
    """
    Benchmark conda MatchSpec filtering all of the linux-64 repodata from
    Anaconda's defaults
    """
    with open(repodata_file) as f:
        repodata = list(json.load(f)['packages'].values())

    benchmark(bench_conda_filter_package_list, list=repodata)
