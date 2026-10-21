# mc

`mc` is a Rust command-line tool that creates Google Forms from YAML templates. It is built for repeatable forms such as workshop registrations, club events, feedback forms, and surveys.

> **Status:** YAML parsing, template validation, Google OAuth login, and form creation from YAML templates are implemented.

## Why mc?

Building recurring forms by hand in the Google Forms web interface is repetitive, and the forms tend to drift apart over time. With `mc`, a form definition lives in a version-controlled YAML file and is created the same way every time from the command line.

## Usage

```bash
mc create form <FILENAME>
```

Example:

```bash
cargo run -- create form examples/workshop.yaml
```

To validate and preview a template without contacting Google:

```bash
cargo run -- create form examples/workshop.yaml --dry-run
```

When run, the command:

1. Checks that the file exists.
2. Parses the YAML template into Rust data structures.
3. Validates the title, questions, options, and question types.
4. Loads the saved Google login and obtains a short-lived access token.
5. Creates the Google Form, then adds its description and questions.
6. Prints the responder URL.

With `--dry-run`, it stops after step 3 and prints a preview.

## Template format

```yaml
title: "AI Agent Development Workshop"
description: "Register for the hands-on weekend workshop."

questions:
  - title: "Full Name"
    type: short_text
    required: true

  - title: "Academic Year"
    type: dropdown
    required: true
    options:
      - "1st Year"
      - "2nd Year"
      - "3rd Year"
      - "4th Year"
```

### Question types

| Type          | Description                    |
|---------------|--------------------------------|
| `short_text`  | Single-line text answer        |
| `paragraph`   | Multi-line text answer         |
| `radio`       | Choose one option              |
| `checkboxes`  | Choose any number of options   |
| `dropdown`    | Choose one option from a list  |

### Template rules

- A form needs a title and at least one question.
- The description is optional.
- `radio`, `checkboxes`, and `dropdown` questions need at least two options.
- Options are not allowed on `short_text` or `paragraph` questions.
- Options cannot be blank or duplicated.

## Development

**Prerequisites:** Rust stable toolchain and Git.

```bash
rustc --version
cargo --version
```

**Run locally:**

```bash
cargo run -- create form examples/workshop.yaml
```

**Quality checks:**

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Project structure

```text
src/
├── main.rs                  # CLI entry point
└── commands/
    ├── mod.rs               # Top-level command routing
    └── create/
        ├── mod.rs           # `mc create` command routing
        └── formhandler.rs   # YAML form-template parsing

examples/
└── workshop.yaml            # Example form template
```

## Roadmap

Phase 1 aims to create a Google Form from a local YAML template. See [Phase 1 Scope](docs/phase-1-scope.md) for the goal, success criteria, and non-goals.

- [x] Define the Phase 1 scope
- [x] Scaffold the Rust CLI
- [x] Parse YAML form templates
- [x] Validate form templates
- [x] Configure Google OAuth
- [x] Create forms through the Google Forms API
- [x] Add questions through the Google Forms API
- [ ] Print edit and responder URLs

Future phases may add an interactive wizard, editing existing forms, sections, and response handling.