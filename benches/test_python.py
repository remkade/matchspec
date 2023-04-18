from conda.models.match_spec import MatchSpec
from pathlib import Path
import rust_matchspec
import json

test_data = Path('./test_data')
depends_file = test_data / 'linux_64-depends.txt'
repodata_file = test_data / 'repodata-linux-64.json'


def test_rust_matchspec_on_repodata_depends(benchmark):
    """
    Test the rust_matchspec.match_against_matchspec using the
    linux_64-depends.txt file.
    """
    def bench_match_against_matchspec(list: [str]):
        for item in list:
            rust_matchspec.match_against_matchspec(item, 'python', '3.9.1')

    with open(depends_file) as f:
        depends = f.readlines()

    benchmark(bench_match_against_matchspec, list=depends)


def test_conda_matchspec_on_repodata_depends(benchmark):
    """
    Test Conda's MatchSpec against the linux_64-depends.txt file
    """
    def bench_conda_against_repodata_depends(list: [str]):
        for item in list:
            ms = MatchSpec(item)
            ms.match({'name': 'python', 'version': '3.9.1',
                     'build': 'hbdb9e5c_0', 'build_number': 0})

    with open(depends_file) as f:
        depends = f.readlines()

    benchmark(bench_conda_against_repodata_depends, list=depends)


def test_rust_matchspec_filter_package_list(benchmark):
    """
    Test rust_matchspec's filter_package_list() against the full linux-64
    repodata.json from Anaconda's defaults.
    """
    def bench_rust_matchspec_filter_package_list(list: [dict[str, str]]):
        _matches = rust_matchspec.filter_package_list('python>=3.9.1', list)

    with open(repodata_file) as f:
        repodata = list(json.load(f)['packages'].values())

    benchmark(bench_rust_matchspec_filter_package_list, list=repodata)


def test_rust_matchspec_parallel_filter_package_list(benchmark):
    """
    Test rust_matchspec's filter_package_list() against the full linux-64
    repodata.json from Anaconda's defaults.
    """
    def bench(list: [dict[str, str]]):
        _matches = rust_matchspec.parallel_filter_package_list(
            'python>=3.9.1', list)

    with open(repodata_file) as f:
        repodata = list(json.load(f)['packages'].values())

    benchmark(bench, list=repodata)


def test_rust_matchspec_parallel_filter_package_list_with_matchspec_list(benchmark):
    """
    Test rust_matchspec's filter_package_list() against the full linux-64
    repodata.json from Anaconda's defaults.
    """
    def bench(list: [dict[str, str]]):
        ms = ['python>=3.9.1', 'openssl>1', 'tensorflow<2.0', 'pip']
        _m = rust_matchspec.parallel_filter_package_list_with_matchspec_list(
            ms, list)

    with open(repodata_file) as f:
        repodata = list(json.load(f)['packages'].values())

    benchmark(bench, list=repodata)


def test_conda_filter_package_list(benchmark):
    """
    Benchmark conda MatchSpec filtering all of the linux-64 repodata from
    Anaconda's defaults
    """

    def bench_conda_filter_package_list(list: [dict[str, str]]):
        ms = MatchSpec('python>=3.9.1')
        _matches = [p for p in list if ms.match(p)]

    with open(repodata_file) as f:
        repodata = list(json.load(f)['packages'].values())

    benchmark(bench_conda_filter_package_list, list=repodata)
