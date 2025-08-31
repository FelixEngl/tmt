# Topic Model Translation
A tool to translate topic models between languages written in rust for python.

> Please Note: Currently the code is broken due some API changes in the dependencies and heavy API changes. I'll will fix the issues in the upcomming months.
> 
> Furthermore: This branch will be merged with the newer branch after applying the appropiate fixes.

This is experimental code and will change over the time.

## How to build?
See: https://pyo3.rs/v0.21.2/

````commandline
python -m venv .env
source .env/bin/activate
pip install maturin
maturin build --release
````

## Links
See more at https://github.com/FelixEngl/ptmt
