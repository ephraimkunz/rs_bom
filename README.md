# rs-bom
![CI](https://github.com/ephraimkunz/rs_bom/workflows/CI/badge.svg)
![Security audit](https://github.com/ephraimkunz/rs_bom/workflows/Security%20audit/badge.svg)

## Getting Started
1. Clone the project.
2. Inside the project directory, run `cargo run` to execute the binary target.
3. Run `cargo doc --open` to generate the docs.
4. Run `cargo test` to run tests.
5. Run `cargo fuzz run reference` to run the reference parser fuzzer.
6. Run `cargo bench` to benchmark.

## Features
* Iter over all the verses in the Book of Mormon. 
* Fetch standalone verses by reference.
* Parse arbitrary reference strings using the format specified [here](https://en.wikipedia.org/wiki/Bible_citation). Canonicalize these references and iterate over the verses in them. For example, given a string of `Alma 3:18–19, 16–17; Alma 3; Alma 4` we can canonicalize it to `Alma 3–4`. Similarly, we canonicalize `Alma 16, 18, 19` to `Alma 16, 18–19`.

## Planned features
* JSON RESTful API for all of the same.
