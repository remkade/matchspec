from .rust_matchspec import *
from typing import Any

__name__ = "rust_matchspec"
__version__ = "0.2.1"


def package_candidate_to_dict(self) -> dict[str, Any]:
    return {
        "name": self.name,
        "version": self.version,
        "build_number": self.build_number,
        "depends": self.depends,
        "license": self.license,
        "md5": self.md5,
        "sha256": self.sha256,
        "size": self.size,
        "subdir": self.subdir,
        "timestamp": self.timestamp,
    }


PackageCandidate.to_dict = package_candidate_to_dict
