"""
OpenAPI spec tasks for the Brokkr broker.

The committed `openapi/brokkr-v1.json` is the contract that downstream SDK
generators consume. These tasks keep it in sync with the broker's
`utoipa`-derived schema:

- `angreal openapi export` regenerates the spec from the broker crate.
- `angreal openapi check` regenerates into a temp file and fails if it
  differs from the committed copy (used by CI for drift detection).
- `angreal openapi gen-python` regenerates the Python SDK from the spec
  using a pinned version of `openapi-python-client` invoked via `uvx`.
"""

import filecmp
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

import angreal

PROJECT_ROOT = Path(angreal.get_root()).parent
SPEC_PATH = PROJECT_ROOT / "openapi" / "brokkr-v1.json"
# Mirror of the spec that ships inside the brokkr-client crate. The
# `progenitor::generate_api!` macro reads from a path *inside* the crate
# directory so the spec survives `cargo package`. `export` keeps the two
# copies byte-identical; `check` asserts they haven't drifted.
CRATE_SPEC_PATH = (
    PROJECT_ROOT / "crates" / "brokkr-client" / "spec" / "brokkr-v1.json"
)
EXPORT_CMD = [
    "cargo",
    "run",
    "--quiet",
    "-p",
    "brokkr-broker",
    "--example",
    "openapi_export",
]

# Pinned generator versions. Bumps are a deliberate maintenance action: the
# generated SDK source is committed, and the diff between versions is the
# review surface.
OPENAPI_PYTHON_CLIENT_VERSION = "0.28.4"
OPENAPI_TYPESCRIPT_VERSION = "7.13.0"
PYTHON_SDK_DIR = PROJECT_ROOT / "sdks" / "python" / "brokkr-client"
PYTHON_GEN_CONFIG = (
    Path(angreal.get_root()) / "files" / "openapi-python-client-config.yaml"
)
TYPESCRIPT_SDK_DIR = PROJECT_ROOT / "sdks" / "typescript" / "brokkr-client"
TYPESCRIPT_SCHEMA_REL = "src/schema.d.ts"

openapi = angreal.command_group(name="openapi", about="OpenAPI spec tasks")


def _run_export(out_path: Path) -> int:
    """Run the `openapi_export` cargo example, writing the result to out_path.

    The example writes to `<workspace>/openapi/brokkr-v1.json` unconditionally,
    so we run it, then move the artifact to `out_path` if needed.
    """
    env_msg = f"writing to {out_path}"
    print(f"Regenerating OpenAPI spec ({env_msg})...")
    try:
        subprocess.run(EXPORT_CMD, cwd=str(PROJECT_ROOT), check=True)
    except FileNotFoundError:
        print("cargo not found on PATH", file=sys.stderr)
        return 1
    except subprocess.CalledProcessError as e:
        print(f"openapi_export failed: {e}", file=sys.stderr)
        return e.returncode

    generated = PROJECT_ROOT / "openapi" / "brokkr-v1.json"
    if not generated.exists():
        print(f"expected {generated} after export; not found", file=sys.stderr)
        return 1

    if out_path.resolve() != generated.resolve():
        out_path.parent.mkdir(parents=True, exist_ok=True)
        out_path.write_bytes(generated.read_bytes())
    return 0


def _mirror_crate_spec() -> None:
    """Copy SPEC_PATH to CRATE_SPEC_PATH so the brokkr-client crate ships
    the same spec the workspace publishes from."""
    CRATE_SPEC_PATH.parent.mkdir(parents=True, exist_ok=True)
    CRATE_SPEC_PATH.write_bytes(SPEC_PATH.read_bytes())


@openapi()
@angreal.command(
    name="export",
    about="regenerate openapi/brokkr-v1.json from the broker schema",
)
def export():
    """Regenerate `openapi/brokkr-v1.json` from the broker's utoipa schema.

    Also mirrors the spec to `crates/brokkr-client/spec/brokkr-v1.json` so
    the in-crate `progenitor::generate_api!` macro reads from a path that
    survives `cargo package`.
    """
    rc = _run_export(SPEC_PATH)
    if rc == 0:
        _mirror_crate_spec()
        print(
            f"Mirrored spec to {CRATE_SPEC_PATH.relative_to(PROJECT_ROOT)}"
        )
    return rc


@openapi()
@angreal.command(
    name="check",
    about="fail if openapi/brokkr-v1.json is stale relative to the broker schema",
)
def check():
    """Drift check used in CI. Returns non-zero on diff."""
    if not SPEC_PATH.exists():
        print(
            f"committed spec not found at {SPEC_PATH} — run `angreal openapi export`",
            file=sys.stderr,
        )
        return 1

    # Crate-local mirror must exist and match the workspace copy.
    if not CRATE_SPEC_PATH.exists():
        print(
            f"crate spec mirror not found at {CRATE_SPEC_PATH} — "
            f"run `angreal openapi export`",
            file=sys.stderr,
        )
        return 1
    if not filecmp.cmp(SPEC_PATH, CRATE_SPEC_PATH, shallow=False):
        print(
            f"FAIL: {CRATE_SPEC_PATH.relative_to(PROJECT_ROOT)} drifted from "
            f"{SPEC_PATH.relative_to(PROJECT_ROOT)}.\n"
            f"Regenerate locally with: angreal openapi export",
            file=sys.stderr,
        )
        return 1

    with tempfile.TemporaryDirectory() as tmp:
        candidate = Path(tmp) / "brokkr-v1.json"
        # _run_export writes to the workspace location first, then mirrors to candidate.
        # To avoid clobbering the committed copy mid-check we back it up and restore.
        backup = SPEC_PATH.read_bytes()
        try:
            rc = _run_export(candidate)
        finally:
            SPEC_PATH.write_bytes(backup)
        if rc != 0:
            return rc

        if filecmp.cmp(candidate, SPEC_PATH, shallow=False):
            print(f"OK: {SPEC_PATH.relative_to(PROJECT_ROOT)} is up to date")
            return 0

        print(
            f"FAIL: {SPEC_PATH.relative_to(PROJECT_ROOT)} is stale.\n"
            f"Regenerate locally with: angreal openapi export",
            file=sys.stderr,
        )
        try:
            subprocess.run(
                ["diff", "-u", str(SPEC_PATH), str(candidate)],
                cwd=str(PROJECT_ROOT),
                check=False,
            )
        except FileNotFoundError:
            pass
        return 1


def _run_python_gen(target_dir: Path) -> int:
    """Invoke `openapi-python-client generate` writing into target_dir's parent.

    The generator always writes into `<cwd>/<output-path>`, so we run from a
    chosen working dir. The target_dir is expected to be `<wd>/brokkr-client`.
    """
    if not SPEC_PATH.exists():
        print(
            f"spec not found at {SPEC_PATH} — run `angreal openapi export` first",
            file=sys.stderr,
        )
        return 1
    if shutil.which("uvx") is None:
        print(
            "uvx not found on PATH. Install uv: https://docs.astral.sh/uv/",
            file=sys.stderr,
        )
        return 1

    wd = target_dir.parent
    wd.mkdir(parents=True, exist_ok=True)
    name = target_dir.name
    cmd = [
        "uvx",
        f"openapi-python-client@{OPENAPI_PYTHON_CLIENT_VERSION}",
        "generate",
        "--path",
        str(SPEC_PATH),
        "--meta",
        "uv",
        "--output-path",
        name,
        "--overwrite",
        "--config",
        str(PYTHON_GEN_CONFIG),
    ]
    try:
        subprocess.run(cmd, cwd=str(wd), check=True)
    except subprocess.CalledProcessError as e:
        print(f"openapi-python-client failed: {e}", file=sys.stderr)
        return e.returncode
    return 0


@openapi()
@angreal.command(
    name="gen-python",
    about="regenerate sdks/python/brokkr-client from openapi/brokkr-v1.json",
)
def gen_python():
    """Regenerate the Python SDK using a pinned openapi-python-client via uvx.

    Invokes:
        uvx openapi-python-client@<pinned> generate --path <spec> \
            --meta uv --output-path brokkr-client --overwrite

    inside `sdks/python/`. Committed output is the review surface for spec
    changes — review the diff before merging.
    """
    if not SPEC_PATH.exists():
        print(
            f"spec not found at {SPEC_PATH} — run `angreal openapi export` first",
            file=sys.stderr,
        )
        return 1

    if shutil.which("uvx") is None:
        print(
            "uvx not found on PATH. Install uv: https://docs.astral.sh/uv/",
            file=sys.stderr,
        )
        return 1

    print(
        f"Regenerating Python SDK with openapi-python-client@{OPENAPI_PYTHON_CLIENT_VERSION} ..."
    )
    rc = _run_python_gen(PYTHON_SDK_DIR)
    if rc == 0:
        print(f"Wrote Python SDK to {PYTHON_SDK_DIR.relative_to(PROJECT_ROOT)}")
    return rc


def _generated_drifts(fresh: Path, committed: Path) -> list[str]:
    """Return paths that drifted between a freshly regenerated tree and the
    committed copy. One-sided: every file produced by the generator must
    exist identically under `committed`; extras under `committed` (hand-added
    tests, dev-tool caches, etc.) are intentional and ignored.

    Skips internal artifacts of the generator itself (`.ruff_cache`) — those
    are timestamped cache dirs the generator writes during a run, not part
    of its API output.
    """
    import filecmp as fc

    skip_dirs = {"__pycache__", ".ruff_cache"}

    diffs: list[str] = []
    for entry in fresh.rglob("*"):
        rel = entry.relative_to(fresh)
        parts = set(rel.parts)
        if parts & skip_dirs or entry.suffix == ".pyc":
            continue
        target = committed / rel
        if entry.is_dir():
            if not target.is_dir():
                diffs.append(f"missing dir: {rel}")
            continue
        if not target.exists():
            diffs.append(f"missing: {rel}")
            continue
        if not fc.cmp(entry, target, shallow=False):
            diffs.append(f"differs: {rel}")
    return diffs


def _run_typescript_gen(out_path: Path) -> int:
    """Invoke `npx openapi-typescript <spec> -o <out>`."""
    if not SPEC_PATH.exists():
        print(
            f"spec not found at {SPEC_PATH} — run `angreal openapi export` first",
            file=sys.stderr,
        )
        return 1
    if shutil.which("npx") is None:
        print(
            "npx not found on PATH. Install Node.js >= 18.",
            file=sys.stderr,
        )
        return 1

    out_path.parent.mkdir(parents=True, exist_ok=True)
    cmd = [
        "npx",
        "--yes",
        f"openapi-typescript@{OPENAPI_TYPESCRIPT_VERSION}",
        str(SPEC_PATH),
        "-o",
        str(out_path),
    ]
    try:
        subprocess.run(cmd, check=True)
    except subprocess.CalledProcessError as e:
        print(f"openapi-typescript failed: {e}", file=sys.stderr)
        return e.returncode
    return 0


@openapi()
@angreal.command(
    name="gen-typescript",
    about="regenerate sdks/typescript/brokkr-client/src/schema.d.ts from the spec",
)
def gen_typescript():
    """Regenerate the TypeScript SDK's schema.d.ts using openapi-typescript."""
    target = TYPESCRIPT_SDK_DIR / TYPESCRIPT_SCHEMA_REL
    print(
        f"Regenerating TypeScript schema with openapi-typescript@{OPENAPI_TYPESCRIPT_VERSION} ..."
    )
    rc = _run_typescript_gen(target)
    if rc == 0:
        print(f"Wrote TypeScript schema to {target.relative_to(PROJECT_ROOT)}")
    return rc


@openapi()
@angreal.command(
    name="check-typescript",
    about="fail if sdks/typescript/brokkr-client/src/schema.d.ts is stale",
)
def check_typescript():
    """Drift check for the committed TypeScript schema. Used in CI."""
    committed = TYPESCRIPT_SDK_DIR / TYPESCRIPT_SCHEMA_REL
    if not committed.exists():
        print(
            f"committed schema not found at {committed} — run `angreal openapi gen-typescript`",
            file=sys.stderr,
        )
        return 1

    with tempfile.TemporaryDirectory() as tmp:
        candidate = Path(tmp) / "schema.d.ts"
        print(
            f"Regenerating TypeScript schema into temp dir for drift check ({OPENAPI_TYPESCRIPT_VERSION})..."
        )
        rc = _run_typescript_gen(candidate)
        if rc != 0:
            return rc

        if filecmp.cmp(candidate, committed, shallow=False):
            print(
                f"OK: {committed.relative_to(PROJECT_ROOT)} matches the spec"
            )
            return 0

        print(
            f"FAIL: {committed.relative_to(PROJECT_ROOT)} is stale.\n"
            f"Regenerate locally with: angreal openapi gen-typescript",
            file=sys.stderr,
        )
        try:
            subprocess.run(
                ["diff", "-u", str(committed), str(candidate)],
                cwd=str(PROJECT_ROOT),
                check=False,
            )
        except FileNotFoundError:
            pass
        return 1


@openapi()
@angreal.command(
    name="check-python",
    about="fail if sdks/python/brokkr-client is stale relative to the spec",
)
def check_python():
    """Drift check for the committed Python SDK. Used in CI.

    Regenerates the SDK into a temp directory using the same pinned generator
    as `gen-python`, then walks the committed tree side-by-side and reports
    any files that differ, were added, or were removed. Ignores `__pycache__`
    and `*.pyc`.
    """
    if not PYTHON_SDK_DIR.exists():
        print(
            f"committed SDK not found at {PYTHON_SDK_DIR} — run `angreal openapi gen-python`",
            file=sys.stderr,
        )
        return 1

    with tempfile.TemporaryDirectory() as tmp:
        candidate = Path(tmp) / "brokkr-client"
        print(
            f"Regenerating Python SDK into temp dir for drift check ({OPENAPI_PYTHON_CLIENT_VERSION})..."
        )
        rc = _run_python_gen(candidate)
        if rc != 0:
            return rc

        diffs = _generated_drifts(candidate, PYTHON_SDK_DIR)
        if not diffs:
            print(
                f"OK: {PYTHON_SDK_DIR.relative_to(PROJECT_ROOT)} matches the spec"
            )
            return 0

        print(
            f"FAIL: {PYTHON_SDK_DIR.relative_to(PROJECT_ROOT)} is stale "
            f"({len(diffs)} drifted entries).\n"
            f"Regenerate locally with: angreal openapi gen-python",
            file=sys.stderr,
        )
        for d in diffs[:50]:
            print(f"  {d}", file=sys.stderr)
        if len(diffs) > 50:
            print(f"  ... and {len(diffs) - 50} more", file=sys.stderr)
        return 1
