# mc — Phase 1 Scope

## Goal

Create a Google Form from a local YAML template.

## Primary user flow

The user runs:

```bash
mc create --template examples/workshop.yaml
```


## The CLI

1. Loads the YAML File.
2. Validates its structures and values.
3. Authenticates with the user's google account when needed.
4. Creates a Google form.
5. Adds the template's description and questions .
6. Prints the form ID, edit URL, and the responder URL


## Supported template fields

Form title
Form description
Question title
Question type
Required flag
Choice options


## Supported question types

short_text
paragraph
radio
checkboxes
dropdown

## Commands in Phase 1 

```bash
mc create form <FILENAME>
mc create form <FILENAME> --dry-run
mc auth login
mc auth status
mc auth logout
```