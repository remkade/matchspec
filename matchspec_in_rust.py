import rust_matchspec

def match_against_matchspec(matchspec: str, package: str, version: str) -> bool:
    return rust_matchspec.match_against_matchspec(matchspec, package, version)
