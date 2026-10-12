# Console screenshot harness

`shots.mjs` opens each view of the operator console in a headless browser and
takes a screenshot. It mocks the broker API, so no broker is necessary. It
fails when a scene has a console error or a failed assert.

## Run it

```sh
cd crates/brokkr-web/web-e2e
npm ci        # one time: installs Playwright and its browser
./run.sh      # serves a release build, then runs all scenes, dark and light
```

The screenshots go to `shots/dark/` and `shots/light/`. The `trunk serve` log
goes to `shots/trunk.log`.

Options (environment variables):

| Variable | Default | Use |
|---|---|---|
| `ONLY` | all scenes | A regex. Runs only the scenes whose names match. |
| `THEMES` | `dark light` | The themes to run. |
| `PORT` | `9080` | The port for `trunk serve`. |
| `OUT_ROOT` | `shots` | The folder for the screenshots. |

To run against a console that you serve yourself:

```sh
URL=http://127.0.0.1:9080 THEME=light OUT=shots/light node shots.mjs
```

## Toolchain and browser

- `../rust-toolchain.toml` sets the Rust version and adds the
  `wasm32-unknown-unknown` target. `cargo` and `trunk` in `crates/brokkr-web`
  use it with no `+toolchain` flag. rustup installs it when it is missing.
- `package.json` pins Playwright to one version. That version sets the browser
  build.
- `npm ci` installs the browser into `node_modules` (the `postinstall` script,
  with `PLAYWRIGHT_BROWSERS_PATH=0`). `shots.mjs` uses the same setting. The
  browser is not in the shared user cache, because a `playwright install` in
  another project removes the browsers of each Playwright install that it
  cannot find, for example one in a removed worktree. Each checkout downloads
  the browser one time (about 200 MB).
