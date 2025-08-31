# Topic Model Translation
A tool to translate topic models between languages written in rust for python.

This is experimental code and will change over the time.

> Please Note: Currently the code is broken due some API changes in the dependencies and heavy API changes. I'll will fix the issues in the upcomming months.

## How to build?
See: https://pyo3.rs/v0.22.5/

````commandline
python -m venv .env
source .env/bin/activate
pip install maturin
maturin build --release
````


build pyi:
````commandline
cargo run --features gen_python_api --bin stub_gen
````

## Links
See more at https://github.com/FelixEngl/ptmt


## Known problems:
- using gen_stub_pymethods on enum impls results in a panic.
