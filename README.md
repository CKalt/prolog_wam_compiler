# Prolog WAM Compiler in Rust

This project is a Prolog compiler targeting the Warren Abstract Machine (WAM) implemented in Rust.

## Project Structure

The project is organized in a modular fashion with the following directory structure:

<pre>
prolog_wam_compiler/
├── Cargo.toml
└── src
    ├── main.rs
    ├── lib.rs
    ├── parser
    │   ├── mod.rs
    │   ├── term.rs
    │   ├── clause.rs
    │   └── grammar.rs
    ├── wam
    │   ├── mod.rs
    │   ├── emulator.rs
    │   ├── instruction.rs
    │   ├── error.rs
    │   └── data_structures.rs
    └── runtime
        ├── mod.rs
        ├── builtins.rs
        ├── io.rs
        └── utils.rs
</pre>

### Description of Files and Directories

- `Cargo.toml`: Contains project metadata, dependencies, and build settings.
- `src/main.rs`: Contains the main function for the command-line interface (CLI) tool.
- `src/lib.rs`: Exports modules and serves as the entry point for the library.
- `src/parser/`: Contains files related to the Prolog parser.
  - `mod.rs`: Exports parser components.
  - `term.rs`: Implements parsing of Prolog terms.
  - `clause.rs`: Implements parsing of Prolog clauses.
  - `grammar.rs`: Contains the
- `src/wam/`: Contains files related to the WAM emulator.
  - `mod.rs`: Exports WAM emulator components.
  - `emulator.rs`: Implements the core functionality of the WAM emulator.
  - `instruction.rs`: Implements WAM instructions.
  - `error.rs`: Defines error types for the WAM emulator.
  - `data_structures.rs`: Contains data structures used by the WAM emulator.
- `src/runtime/`: Contains files related to the runtime system.
  - `mod.rs`: Exports runtime components.
  - `builtins.rs`: Implements Prolog built-in predicates.
  - `io.rs`: Handles input/output operations for the runtime system.
  - `utils.rs`: Contains utility functions and data structures for the runtime system.

## Current Progress

This project currently implements a minimal WAM emulator with basic heap functionality. The emulator can push Prolog terms onto the heap, and the storage can be verified using Rust's testing framework.

## Getting Started

To build and run the project, you'll need to have Rust and Cargo installed. Follow the instructions on the official Rust website to install them: https://www.rust-lang.org/tools/install 

Once Rust and Cargo are installed, you can use the `cargo` command-line tool to build, run, and test your project 
by following these commands in your project's root directory (the one containing `Cargo.toml`):

1. **Build the project**: Run the following command to build your project:

   <pre>
   cargo build
   </pre>

2. **Run the project**: After building the project, you can run it using the following command:

   <pre>
   cargo run
   </pre>

3. **Run the tests**: To run the test cases in your project, use the following command:

   <pre>
   cargo test
   </pre>

## Contributing

To contribute to this project, you can fork the repository on GitHub, create a new branch, and submit a pull request with your changes.

## License

This project is released under the MIT License.


# prolog_wam_compiler
