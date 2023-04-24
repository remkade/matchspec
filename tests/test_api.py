import pytest
from rust_matchspec import PackageCandidate


class TestPackageCandidate:
    def test_creation_with_valid_dicts(self):
        """
        Create PackageCandidate from dict and make sure attrs are set
        """
        pc = PackageCandidate.from_dict(
            {'name': 'tensorflow', 'version': '2.12.0'})
        assert pc.name == 'tensorflow'
        assert pc.version == '2.12.0'
        assert pc.license is None

    def test_package_candidate_creation_with_invalid_dict(self):
        """
        Expect the PackageCandidate to fail because a missing 'name' key in the
        dict
        """
        with pytest.raises(KeyError):
            PackageCandidate.from_dict({})

    def test_roundtrip_from_dict(self):
        """
        Create from dict return to dict
        """
        pc = PackageCandidate.from_dict({'name': 'tensorflow',
                                         'version': '2.12.0'})
        d = pc.to_dict()

        assert d['name'] == 'tensorflow'
        assert d['version'] == '2.12.0'
        # These keys should exist but default to None
        assert d['license'] is None
        assert d['depends'] == []
