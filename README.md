# mc

`mc` is a Rust command-line tool for creating Google Forms from YAML templates. It is designed for repeatable forms such as workshop registrations, club events, feedback forms, and surveys.

> Project status: YAML parsing, template validation, and Google OAuth login are implemented. Google Forms API creation is next.
## Why mc?

Building recurring forms manually in the Google Forms web interface is repetitive and makes it easy for forms to drift apart. With `mc`, a form definition can live in a version-controlled YAML file and be created consistently from the command line.

## Current command

```bash
mc create form <FILENAME>
```

For example:

```bash
cargo run -- create form examples/workshop.yaml
```

The command currently:

1. Checks that the provided file exists.
2. Parses the YAML template into Rust form data structures.
3. Prints the parsed form data for development and verification.

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

### Supported question types

- `short_text`
- `paragraph`
- `radio`
- `checkboxes`
- `dropdown`

### Template rules

- Each form has a title and one or more questions.
- The description is optional.
- `radio`, `checkboxes`, and `dropdown` questions will require options when validation is added.
- Options are not used by `short_text` or `paragraph` questions.

## Development setup

### Prerequisites

- Rust stable toolchain
- Git

Verify the Rust installation:

```bash
rustc --version
cargo --version
```

### Run locally

```bash
cargo run -- create form examples/workshop.yaml
```

### Quality checks

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Roadmap

Phase 1 aims to create a Google Form from a local YAML template.

- [x] Define the Phase 1 scope
- [x] Scaffold the Rust CLI
- [x] Parse YAML form templates
- [x] Validate form templates
- [x] Configure Google OAuth
- [ ] Create forms through the Google Forms API
- [ ] Add questions through the Google Forms API
- [ ] Print edit and responder URLs

Future phases may add an interactive wizard, editing existing forms, sections, and response handling.

## Project structure

```text
src/
├── main.rs                    # CLI entry point
└── commands/
    ├── mod.rs                 # Top-level command routing
    └── create/
        ├── mod.rs             # `mc create` command routing
        └── formhandler.rs     # YAML form-template parsing

examples/
└── workshop.yaml              # Example form template
```

## Phase 1 scope

See [Phase 1 Scope](docs/phase-1-scope.md) for the defined goal, success criteria, and explicit non-goals.
