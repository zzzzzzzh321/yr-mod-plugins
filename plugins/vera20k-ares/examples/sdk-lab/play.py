"""Play the authored SDK lab through VERA20K's existing community launch pipe."""
from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import struct
import subprocess
import tempfile


HERE = Path(__file__).resolve().parent
MODES = ("vanilla", "compatible", "overload")


def make_frame(map_path: Path, spawn_path: Path, component: Path | None, mode: str) -> bytes:
    if mode not in MODES:
        raise ValueError("unknown test mode")
    text = map_path.read_text(encoding="ascii")
    text, changes = re.subn(
        r"(?m)^Overload=(?:yes|no)$",
        "Overload=" + ("yes" if mode == "overload" else "no"),
        text,
    )
    if changes != 1:
        raise ValueError("test map must have exactly one Overload switch")
    files = [("sdk-feature-lab.map", text.encode("ascii"))]
    if mode != "vanilla":
        if component is None:
            raise ValueError("component is required for this mode")
        declaration = component.with_name("vera-mod.json").read_bytes()
        manifest = json.loads(declaration)
        wasm = component.read_bytes()
        if manifest["api_version"] != 1 or not wasm.startswith(b"\0asm"):
            raise ValueError("unsupported component declaration or Wasm header")
        if hashlib.sha256(wasm).hexdigest() != manifest["sha256"]:
            raise ValueError("component digest disagrees with vera-mod.json")
        name = manifest["program"]
        if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9_.-]{0,127}", name):
            raise ValueError("component resource name is not portable")
        if len(wasm) > 16 * 1024 * 1024:
            raise ValueError("component exceeds the engine resource limit")
        files.extend([("vera-mod.json", declaration), (name, wasm)])
    payload = json.dumps(
        {
            "protocol": 1,
            "release": "sdk-feature-lab-" + mode,
            "spawn": spawn_path.read_text(encoding="ascii"),
            "resources": [
                {"name": name, "data": base64.b64encode(data).decode("ascii")}
                for name, data in files
            ],
        },
        ensure_ascii=True,
        separators=(",", ":"),
    ).encode("utf-8")
    if len(payload) > 48 * 1024 * 1024:
        raise ValueError("launch frame exceeds the engine limit")
    return struct.pack("<I", len(payload)) + payload


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--engine", type=Path, required=True)
    parser.add_argument("--game-dir", type=Path, required=True)
    parser.add_argument("--component", type=Path)
    parser.add_argument("--mode", choices=MODES, default="overload")
    parser.add_argument("--map", type=Path, default=HERE / "sdk-feature-lab.map")
    parser.add_argument("--spawn", type=Path, default=HERE / "spawn.ini")
    parser.add_argument("--work-dir", type=Path, default=Path(tempfile.gettempdir()) / "vera-sdk-lab")
    parser.add_argument("--detach", action="store_true", help="return after launching the game")
    parser.add_argument("--windowed", action="store_true", help="use a window instead of full screen")
    parser.add_argument("--dry-run", action="store_true", help="validate and describe without starting")
    args = parser.parse_args()
    engine = args.engine.resolve(strict=True)
    game = args.game_dir.resolve(strict=True)
    if not engine.is_file() or not game.is_dir():
        parser.error("engine must be a file and game-dir must be a directory")
    frame = make_frame(args.map, args.spawn, args.component, args.mode)
    if args.dry_run:
        payload = json.loads(frame[4:])
        print(json.dumps({"mode": args.mode, "engine": str(engine), "game": str(game),
                          "resources": [r["name"] for r in payload["resources"]],
                          "frameBytes": len(frame)}, ensure_ascii=False))
        return 0
    args.work_dir.mkdir(parents=True, exist_ok=True)
    run = Path(tempfile.mkdtemp(prefix=args.mode + "-", dir=args.work_dir))
    profile = run / "profile"
    profile.mkdir()
    (profile / "VERA.INI").write_text(
        "[Intro]\nPlay=no\n[VERA]\nZoom=100\nOriginalShape=no\nFullScreen="
        + ("no" if args.windowed else "yes") + "\n", encoding="ascii",
    )
    # Do not import personal gameplay overrides into a reproducible test.
    (profile / "VERARULES.INI").write_text("; SDK feature lab: no personal overrides.\n", encoding="ascii")
    # Paths belong to this private run; retail assets are opened read-only.
    config = "[paths]\nra2_dir = " + json.dumps(game.as_posix())
    config += '\nsettings_dir = "profile"\n\n[gameplay]\npause_on_focus_loss = true\n'
    (run / "config.toml").write_text(config, encoding="utf-8")
    env = dict(os.environ)
    env.pop("RA2_QUICKPLAY", None)
    env.pop("RA2_DEBUG_SPAWN_UNITS", None)
    with (run / "process.log").open("wb") as log:
        child = subprocess.Popen([str(engine), "--community-launch"], cwd=run, env=env,
                                 stdin=subprocess.PIPE, stdout=log, stderr=subprocess.STDOUT)
        assert child.stdin is not None
        try:
            child.stdin.write(frame)
            child.stdin.close()
        except BrokenPipeError:
            child.wait()
            raise RuntimeError(f"engine rejected launch; see {run / 'process.log'}") from None
        print(json.dumps({"pid": child.pid, "mode": args.mode, "runDirectory": str(run)},
                         ensure_ascii=False), flush=True)
        if args.detach:
            return 0
        result = child.wait()
    if result:
        print(f"Engine exit {result}; see {run / 'process.log'}")
    return result


if __name__ == "__main__":
    raise SystemExit(main())
