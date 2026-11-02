"""Focused checks for the lab's launch mode and component identity boundary."""
import base64
import hashlib
import importlib.util
import json
from pathlib import Path
import struct
import tempfile
import unittest

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("sdk_lab_play", HERE / "play.py")
play = importlib.util.module_from_spec(spec)
spec.loader.exec_module(play)


class LabLaunchTests(unittest.TestCase):
    def test_vanilla_omits_component_and_mode_change_preserves_source(self):
        original = (HERE / "sdk-feature-lab.map").read_bytes()
        frame = play.make_frame(HERE / "sdk-feature-lab.map", HERE / "spawn.ini", None, "vanilla")
        self.assertEqual(struct.unpack("<I", frame[:4])[0], len(frame) - 4)
        payload = json.loads(frame[4:])
        self.assertEqual([r["name"] for r in payload["resources"]], ["sdk-feature-lab.map"])
        rendered = base64.b64decode(payload["resources"][0]["data"])
        self.assertIn(b"Overload=no\n", rendered)
        self.assertEqual((HERE / "sdk-feature-lab.map").read_bytes(), original)

    def test_component_digest_mismatch_refuses_to_launch(self):
        with tempfile.TemporaryDirectory() as directory:
            wasm = Path(directory) / "vera20k-ares.wasm"
            wasm.write_bytes(b"\0asm\x0d\0\x01\0")
            declaration = {"api_version": 1, "program": wasm.name, "sha256": "0" * 64}
            wasm.with_name("vera-mod.json").write_text(json.dumps(declaration), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "digest"):
                play.make_frame(HERE / "sdk-feature-lab.map", HERE / "spawn.ini", wasm, "overload")
            declaration["sha256"] = hashlib.sha256(wasm.read_bytes()).hexdigest()
            wasm.with_name("vera-mod.json").write_text(json.dumps(declaration), encoding="utf-8")
            payload = json.loads(play.make_frame(HERE / "sdk-feature-lab.map", HERE / "spawn.ini",
                                                wasm, "overload")[4:])
            self.assertEqual(len(payload["resources"]), 3)


if __name__ == "__main__":
    unittest.main()
