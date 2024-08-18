# brokkr

An environment aware control plane for dynamically distributing kubernetes objects.

## Environment Setup

- _angreal_
```
pip install angreal
```

#### Required Libraries
- _mac_
```bash
brew install libpq
# you might need to do this if diesel fails
export LDFLAGS="-L/opt/homebrew/opt/libpq/lib"
export CPPFLAGS="-I/opt/homebrew/opt/libpq/include"
export PATH="/opt/homebrew/opt/libpq/bin:$PATH"
```

- _linux_
```bash
sudo apt-get install libpq-dev
```

#### Rust Stuff

```bash
cargo install cargo-binstall
cargo binstall diesel_cli
```
