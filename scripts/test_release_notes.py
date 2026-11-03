"""Integration tests run in disposable repositories; no network or signing key."""
from contextlib import contextmanager
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

import changelog
import prepare_release
import publish_release

ROOT = Path(__file__).resolve().parent.parent


@contextmanager
def repository():
    with tempfile.TemporaryDirectory() as directory:
        previous = Path.cwd()
        os.chdir(directory)
        try:
            subprocess.run(["git", "init", "-q"], check=True)
            subprocess.run(["git", "config", "user.email", "test@example.org"], check=True)
            subprocess.run(["git", "config", "user.name", "Test"], check=True)
            for channel, filename in changelog.FILES.items():
                Path(filename).write_text(changelog.History(f"# {channel}", [changelog.pending()]).render(), encoding="utf-8")
            Path("Cargo.toml").write_text('[package]\nname = "aetheris"\nversion = "0.1.0"\n', encoding="utf-8")
            Path("Cargo.lock").write_text('[[package]]\nname = "aetheris"\nversion = "0.1.0"\n', encoding="utf-8")
            subprocess.run(["git", "add", "."], check=True)
            subprocess.run(["git", "commit", "-qm", "chore: initialize"], check=True)
            yield
        finally:
            os.chdir(previous)


def stage_all():
    subprocess.run(["git", "add", "."], check=True)


def add_note(channel, note="- Une nouvelle fonction.", section="Add"):
    path = Path(changelog.FILES[channel])
    history = changelog.parse(path.read_text(encoding="utf-8"), channel)
    history.entries[0].sections[section] = note
    path.write_text(history.render(), encoding="utf-8")


EXEMPT = "\nChangelog-Exempt: stable — aucun effet joueur\nChangelog-Exempt: beta — aucun effet joueur\n"


class CommitTests(unittest.TestCase):
    def test_both_pending_required(self):
        with repository():
            Path("behavior.rs").write_text("changed")
            stage_all()
            with self.assertRaises(ValueError):
                changelog.check_commit("feat: change")
            add_note("beta")
            stage_all()
            with self.assertRaises(ValueError):
                changelog.check_commit("feat: change")
            add_note("stable")
            stage_all()
            changelog.check_commit("feat: change")

    def test_unstaged_notes_do_not_count(self):
        with repository():
            add_note("stable")
            add_note("beta")
            with self.assertRaises(ValueError):
                changelog.check_commit("feat: change")

    def test_exempt_stable_for_beta_fix(self):
        with repository():
            add_note("beta", "- Un problème propre à la bêta est corrigé.", "Fixes")
            stage_all()
            changelog.check_commit("fix: beta\nChangelog-Exempt: stable — bug absent de la stable")

    def test_both_exempt(self):
        with repository():
            changelog.check_commit("docs: internal" + EXEMPT)

    def test_invalid_exemptions(self):
        for trailer in ("Changelog-Exempt: all — docs", "Changelog-Exempt: stable — ", "Changelog-Exempt: beta", "Changelog-Exempt: beta - docs", "changelog-exempt: beta — docs"):
            with self.subTest(trailer=trailer), self.assertRaises(ValueError):
                changelog.exemptions(trailer)

    def test_invalid_staged_structure_even_when_exempt(self):
        with repository():
            Path("CHANGES.md").write_text("invalid")
            stage_all()
            with self.assertRaises(ValueError):
                changelog.check_commit("docs: internal" + EXEMPT)

    def test_history_only_edit_does_not_count(self):
        with repository():
            prepare_release.prepare("1.0.0", "stable")
            stage_all()
            subprocess.run(["git", "commit", "-qm", "chore: release"], check=True)
            path = Path("CHANGES.md")
            path.write_text(path.read_text(encoding="utf-8").replace("# stable", "# Historique"), encoding="utf-8")
            stage_all()
            with self.assertRaises(ValueError):
                changelog.check_commit("fix: something\nChangelog-Exempt: beta — absent de la bêta")

    def test_actual_installed_hook(self):
        with repository():
            shutil.copytree(ROOT / "scripts", "scripts", ignore=shutil.ignore_patterns("__pycache__"))
            shutil.copytree(ROOT / ".githooks", ".githooks")
            subprocess.run([sys.executable, "scripts/install_hooks.py"], check=True, capture_output=True)
            Path("behavior.rs").write_text("changed")
            stage_all()
            rejected = subprocess.run(["git", "commit", "-qm", "feat: change"], capture_output=True)
            self.assertNotEqual(rejected.returncode, 0)
            add_note("beta")
            add_note("stable")
            stage_all()
            subprocess.run(["git", "commit", "-qm", "feat: change"], check=True, capture_output=True)
            prepare_release.prepare("1.0.0", "stable")
            stage_all()
            subprocess.run(["git", "commit", "-qm", "chore(release): prepare v1.0.0" + EXEMPT], check=True, capture_output=True)

    def test_install_preserves_existing_hooks_configuration(self):
        with repository():
            subprocess.run(["git", "config", "core.hooksPath", "custom-hooks"], check=True)
            result = subprocess.run([sys.executable, str(ROOT / "scripts/install_hooks.py")], capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(changelog.git("config", "core.hooksPath").strip(), "custom-hooks")


class ReleaseTests(unittest.TestCase):
    def test_beta_freezes_only_beta(self):
        with repository(), patch("prepare_release.timestamp", return_value="2026-11-03T14:30:00Z"):
            add_note("beta")
            add_note("stable", "- La fonction dans son état final.")
            stable = Path("CHANGES.md").read_bytes()
            subprocess.run(["git", "tag", "v1.2.3-beta.4"], check=True)
            version, notes = prepare_release.prepare("1.2.3", "beta")
            self.assertEqual(version, "1.2.3-beta.5")
            self.assertEqual(Path("CHANGES.md").read_bytes(), stable)
            history = changelog.parse(Path("CHANGES.beta.md").read_text(encoding="utf-8"), "beta")
            self.assertFalse(history.entries[0].has_changes())
            self.assertEqual(notes, history.entries[1].render())
            self.assertIn("1.2.3-beta.5 — 2026-11-03T14:30:00Z", notes)
            for filename in ("Cargo.toml", "Cargo.lock"):
                self.assertIn(version, Path(filename).read_text(encoding="utf-8"))

    def test_stable_clears_pending_and_preserves_beta_history(self):
        with repository():
            add_note("beta")
            prepare_release.prepare("1.0.0", "beta")
            beta_old = changelog.parse(Path("CHANGES.beta.md").read_text(encoding="utf-8"), "beta").entries[1].render()
            add_note("beta", "- Réglage intermédiaire.", "Modify")
            add_note("stable", "- La fonction dans son état final.")
            _, notes = prepare_release.prepare("1.0.0", "stable")
            for channel, filename in changelog.FILES.items():
                self.assertFalse(changelog.parse(Path(filename).read_text(encoding="utf-8"), channel).entries[0].has_changes())
            self.assertIn(beta_old, Path("CHANGES.beta.md").read_text(encoding="utf-8"))
            self.assertIn("état final", notes)
            self.assertNotIn("intermédiaire", notes)

    def test_empty_release_uses_standard_mention(self):
        with repository():
            _, notes = prepare_release.prepare("1.0.0", "stable")
            self.assertIn(changelog.EMPTY, notes)
            changelog.parse(Path("CHANGES.md").read_text(encoding="utf-8"), "stable")

    def test_resume_uses_tag_after_main_moves(self):
        with repository():
            add_note("beta", "- Notes originales.")
            version, notes = prepare_release.prepare("1.0.0", "beta")
            stage_all()
            subprocess.run(["git", "commit", "-qm", "chore: release"], check=True)
            tag = f"v{version}"
            subprocess.run(["git", "tag", tag], check=True)
            tagged = changelog.git("rev-parse", "HEAD").strip()
            branch = changelog.git("symbolic-ref", "--short", "HEAD").strip()
            add_note("beta", "- Future version.")
            stage_all()
            subprocess.run(["git", "commit", "-qm", "feat: next"], check=True)
            with patch("prepare_release.timestamp", side_effect=AssertionError("Must not freeze again")):
                self.assertEqual(prepare_release.prepare("1.0.0", "beta", tag), (version, notes))
            self.assertEqual(changelog.git("rev-parse", "HEAD").strip(), tagged)
            self.assertIn("Future version", changelog.git("show", f"{branch}:CHANGES.beta.md"))
            self.assertEqual(changelog.git("status", "--porcelain"), "")

    def test_workflow_outputs_match_frozen_history(self):
        with repository(), tempfile.TemporaryDirectory() as output_directory:
            add_note("stable", "- Une fonction utile.")
            output = Path(output_directory) / "outputs"
            environment = {
                "RELEASE_VERSION": "1.2.3",
                "RELEASE_CHANNEL": "stable",
                "RELEASE_RESUME_TAG": "",
                "GITHUB_OUTPUT": str(output),
                "RUNNER_TEMP": output_directory,
            }
            with patch.dict(os.environ, environment):
                prepare_release.main()
            values = dict(line.split("=", 1) for line in output.read_text(encoding="utf-8").splitlines())
            self.assertEqual(values["version"], "1.2.3")
            history = changelog.parse(Path("CHANGES.md").read_text(encoding="utf-8"), "stable")
            self.assertEqual(Path(values["notes_path"]).read_text(encoding="utf-8"), history.entries[1].render())

    def test_resume_rejects_tagged_cargo_mismatch(self):
        with repository():
            prepare_release.prepare("1.0.0", "stable")
            Path("Cargo.lock").write_text('[[package]]\nname = "aetheris"\nversion = "2.0.0"\n', encoding="utf-8")
            stage_all()
            subprocess.run(["git", "commit", "-qm", "chore: invalid release"], check=True)
            subprocess.run(["git", "tag", "v1.0.0"], check=True)
            with self.assertRaises(ValueError):
                prepare_release.prepare("1.0.0", "stable", "v1.0.0")

    def test_existing_stable_tag_requires_explicit_resume(self):
        with repository():
            subprocess.run(["git", "tag", "v1.0.0"], check=True)
            with self.assertRaises(ValueError):
                prepare_release.prepare("1.0.0", "stable")

    def test_invalid_requests_do_not_write(self):
        for base, channel, tag in (("01.2.3", "beta", ""), ("1.0.0", "other", ""), ("1.0.0", "stable", "v1.0.0-beta.1"), ("1.0.0", "beta", "v2.0.0-beta.1")):
            with self.subTest(base=base, channel=channel, tag=tag), repository():
                with self.assertRaises(ValueError):
                    prepare_release.prepare(base, channel, tag)
                self.assertEqual(changelog.git("status", "--porcelain"), "")

    def test_legacy_tag_without_frozen_notes_is_rejected(self):
        with repository():
            subprocess.run(["git", "tag", "v0.1.0"], check=True)
            with self.assertRaises(ValueError):
                prepare_release.prepare("0.1.0", "stable", "v0.1.0")

    def test_invalid_other_history_prevents_partial_freeze(self):
        with repository():
            Path("CHANGES.md").write_text("broken")
            before = Path("CHANGES.beta.md").read_bytes()
            with self.assertRaises(ValueError):
                prepare_release.prepare("1.0.0", "beta")
            self.assertEqual(Path("CHANGES.beta.md").read_bytes(), before)
            self.assertIn('version = "0.1.0"', Path("Cargo.toml").read_text(encoding="utf-8"))


class StructureTests(unittest.TestCase):
    def test_malformed_structure(self):
        valid = changelog.History("# stable", [changelog.pending()]).render()
        for invalid in (valid.replace("PENDING", "1.0.0"), valid.replace("### Add", "### Added"), valid.replace("### Fixes", "### Add"), valid + "\n## PENDING\n", valid.replace("### Add\n", "### Add\ntexte\n")):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                changelog.parse(invalid, "stable")

    def test_invalid_date_duplicate_version_wrong_channel(self):
        history = changelog.History("# beta", [changelog.pending()])
        history.freeze("1.0.0-beta.1", "2026-11-03T00:00:00Z")
        valid = history.render()
        for invalid, channel in ((valid.replace("2026-11-03", "2026-02-30"), "beta"), (valid, "stable"), (valid + "\n" + history.entries[1].render(), "beta")):
            with self.subTest(channel=channel), self.assertRaises(ValueError):
                changelog.parse(invalid, channel)


class PublicationTests(unittest.TestCase):
    def invoke(self, response, returncode=0, fail_upload=False):
        calls = []

        def run(args, **kwargs):
            calls.append(args)
            if args[1] == "api":
                return subprocess.CompletedProcess(args, returncode, json.dumps(response), "failure" if returncode else "")
            if fail_upload and args[2] == "upload":
                raise subprocess.CalledProcessError(1, args)
            return subprocess.CompletedProcess(args, 0)

        with tempfile.TemporaryDirectory() as directory:
            notes = Path(directory) / "notes.md"
            notes.write_text("Frozen notes", encoding="utf-8")
            with patch("publish_release.subprocess.run", side_effect=run):
                publish_release.publish("owner/repo", "1.0.0-beta.1", "beta", str(notes), "aetheris-windows-x86_64.zip")
        return calls

    def test_new_release_is_draft_then_uploaded_then_published(self):
        calls = self.invoke({"status": "404"}, 1)
        self.assertEqual([call[2] for call in calls[1:]], ["create", "upload", "edit"])
        self.assertIn("--draft", calls[1])
        self.assertIn("--prerelease", calls[1])
        self.assertIn("--notes-file", calls[1])
        self.assertIn("--draft=false", calls[-1])

    def test_resume_existing_draft(self):
        calls = self.invoke({"draft": True})
        self.assertEqual([call[2] for call in calls[1:]], ["upload", "edit"])
        self.assertIn("--clobber", calls[1])

    def test_already_published_is_noop(self):
        calls = self.invoke({"draft": False, "body": "Frozen notes", "assets": [{"name": "aetheris-windows-x86_64.zip"}]})
        self.assertEqual(len(calls), 1)

    def test_network_failure_does_not_create_release(self):
        with self.assertRaises(ValueError):
            self.invoke({"status": "503"}, 1)

    def test_failed_upload_does_not_publish(self):
        with self.assertRaises(subprocess.CalledProcessError):
            self.invoke({"draft": True}, fail_upload=True)

    def test_published_notes_mismatch_rejected(self):
        with self.assertRaises(ValueError):
            self.invoke({"draft": False, "body": "Other notes"})


if __name__ == "__main__":
    unittest.main()
