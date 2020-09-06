# rs-bom
![CI](https://github.com/ephraimkunz/rs_bom/workflows/CI/badge.svg)
![Security audit](https://github.com/ephraimkunz/rs_bom/workflows/Security%20audit/badge.svg)
![Dependabot](https://flat.badgen.net/dependabot/ephraimkunz/rs_bom?icon=dependabot)
[![codecov](https://codecov.io/gh/ephraimkunz/rs_bom/branch/master/graph/badge.svg)](https://codecov.io/gh/ephraimkunz/rs_bom)

## Getting Started
1. Clone the project.
2. Inside the project directory, run `cargo run --bin rs_bom_cli` to run the command-line app. Run `cargo run --bin rs_bom_api` to start serving the RESTful API. Run `cargo run --bin rs_bom_emailer` to send an email with a random verse. This will need the `USERNAME` and `PASSWORD` environment variables to be specified at build time.
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
* CLI app providing terminal interface to the Book of Mormon.
* Get a random verse
* Search for a reference or for arbitrary text. Limit returned results and get total match count.
* Output all text for consumption for other command-line utilities such as `grep`.

### Crate rs_bom_api
* JSON RESTful API
* Swagger (OpenAPI) documentation
* Get a specific verse
* Canonicalize a reference string
* Get all verses in a reference
* Get a random verse


