# rs-bom
![CI](https://github.com/ephraimkunz/rs_bom/workflows/CI/badge.svg)
![Security audit](https://github.com/ephraimkunz/rs_bom/workflows/Security%20audit/badge.svg)

## Getting Started
1. Clone the project.
2. Inside the project directory, run `cargo run --bin rs_bom_cli` to run the command-line app. Run `cargo run --bin rs_bom_api` to start serving the RESTful API.
3. Run `cargo doc --open` to generate the docs.
4. Run `cargo test` to run tests.
5. Run `cargo fuzz run reference` to run the reference parser fuzzer.
6. Run `cargo bench` to benchmark.

## Features
### Crate rs_bom
* Core functionality of Book of Mormon parsing.
* Iter over all the verses in the Book of Mormon. 
* Fetch standalone verses by reference.
* Parse arbitrary reference strings using the format specified [here](https://en.wikipedia.org/wiki/Bible_citation). Canonicalize these references and iterate over the verses in them. For example, given a string of `Alma 3:18–19, 16–17; Alma 3; Alma 4` we can canonicalize it to `Alma 3–4`. Similarly, we canonicalize `Alma 16, 18, 19` to `Alma 16, 18–19`.

### Crate rs_bom_cli
* CLI app providing rich terminal interface to the Book of Mormon.

### Crate rs_bom_api
* JSON RESTful API for all functionality exposed by `rs_bom`.
* Swagger (OpenAPI) documentation built-in.


