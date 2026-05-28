import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..'))
import patch_badges


class TestCoverageColor(unittest.TestCase):
    def test_95_is_brightgreen(self):
        self.assertEqual(patch_badges.coverage_color(95.0), 'brightgreen')

    def test_90_is_brightgreen(self):
        self.assertEqual(patch_badges.coverage_color(90.0), 'brightgreen')

    def test_89_is_green(self):
        self.assertEqual(patch_badges.coverage_color(89.9), 'green')

    def test_80_is_green(self):
        self.assertEqual(patch_badges.coverage_color(80.0), 'green')

    def test_79_is_yellow(self):
        self.assertEqual(patch_badges.coverage_color(79.9), 'yellow')

    def test_70_is_yellow(self):
        self.assertEqual(patch_badges.coverage_color(70.0), 'yellow')

    def test_69_is_orange(self):
        self.assertEqual(patch_badges.coverage_color(69.9), 'orange')

    def test_60_is_orange(self):
        self.assertEqual(patch_badges.coverage_color(60.0), 'orange')

    def test_59_is_red(self):
        self.assertEqual(patch_badges.coverage_color(59.9), 'red')


_SAMPLE_README = """\
# forgottenserver-rust

![tests](https://img.shields.io/badge/tests-6255%20passing-brightgreen?style=flat-square)
![coverage](https://img.shields.io/badge/coverage-94.85%25-brightgreen?style=flat-square)
![crates](https://img.shields.io/badge/crates-13-orange?style=flat-square&logo=rust&logoColor=white)
![loc](https://img.shields.io/badge/rust%20LOC-136k-lightgrey?style=flat-square)
"""


class TestPatchReadme(unittest.TestCase):
    def _tmp(self, content=_SAMPLE_README):
        f = tempfile.NamedTemporaryFile(mode='w', suffix='.md', delete=False)
        f.write(content)
        f.close()
        return f.name

    def test_patches_tests_badge(self):
        path = self._tmp()
        try:
            patch_badges.patch_readme(path, 7000, 96.12, '140k', 14)
            with open(path) as fh:
                content = fh.read()
            self.assertIn('badge/tests-7000%20passing-brightgreen', content)
        finally:
            os.unlink(path)

    def test_patches_coverage_badge_value(self):
        path = self._tmp()
        try:
            patch_badges.patch_readme(path, 100, 94.85, '10k', 5)
            with open(path) as fh:
                content = fh.read()
            self.assertIn('badge/coverage-94.85%25-brightgreen', content)
        finally:
            os.unlink(path)

    def test_patches_loc_badge(self):
        path = self._tmp()
        try:
            patch_badges.patch_readme(path, 100, 90.0, '150k', 13)
            with open(path) as fh:
                content = fh.read()
            self.assertIn('badge/rust%20LOC-150k-lightgrey', content)
        finally:
            os.unlink(path)

    def test_patches_crates_badge(self):
        path = self._tmp()
        try:
            patch_badges.patch_readme(path, 100, 90.0, '136k', 15)
            with open(path) as fh:
                content = fh.read()
            self.assertIn('badge/crates-15-orange', content)
        finally:
            os.unlink(path)

    def test_coverage_color_yellow_reflected_in_badge(self):
        path = self._tmp()
        try:
            patch_badges.patch_readme(path, 100, 75.0, '100k', 10)
            with open(path) as fh:
                content = fh.read()
            self.assertIn('badge/coverage-75.00%25-yellow', content)
        finally:
            os.unlink(path)

    def test_zero_tests_uses_red(self):
        path = self._tmp()
        try:
            patch_badges.patch_readme(path, 0, 90.0, '136k', 13)
            with open(path) as fh:
                content = fh.read()
            self.assertIn('badge/tests-0%20passing-red', content)
        finally:
            os.unlink(path)

    def test_missing_badge_raises_system_exit(self):
        path = self._tmp('# No badges here\n')
        try:
            with self.assertRaises(SystemExit):
                patch_badges.patch_readme(path, 100, 90.0, '10k', 5)
        finally:
            os.unlink(path)

    def test_file_unchanged_when_values_identical(self):
        path = self._tmp()
        try:
            with open(path) as fh:
                original = fh.read()
            patch_badges.patch_readme(path, 6255, 94.85, '136k', 13)
            with open(path) as fh:
                updated = fh.read()
            self.assertEqual(original, updated)
        finally:
            os.unlink(path)
