# Easy-Version VCS CLI - Technical Readme

Welcome to the Easy-Version Version Control System (VCS) Command-Line Interface (CLI) tool. This application is designed to be extremely easy to use while providing efficient version control tailored for artists.

## Table of Contents

- [Easy-Version VCS CLI - Technical Readme](#easy-version-vcs-cli---technical-readme)
  - [Table of Contents](#table-of-contents)
  - [Introduction](#introduction)
  - [Installation](#installation)
  - [Commands Overview](#commands-overview)
    - [`save`](#save)
    - [`load`](#load)
    - [`delete`](#delete)
    - [`split`](#split)
    - [`list`](#list)
    - [`label`](#label)
    - [`comment`](#comment)
    - [`diff`](#diff)
    - [`status`](#status)
    - [`help`](#help)
    - [`export`](#export)
    - [`undo`](#undo)
  - [Usage Examples](#usage-examples)
  - [Notes and Considerations](#notes-and-considerations)
  - [Contact and Support](#contact-and-support)

---

## Introduction

This CLI tool allows artists to:

- **Save** versions of their work with optional messages.
- **Load** previous versions easily.
- **Delete** versions safely.
- **Split** projects into new ones based on existing versions.
- **Label** and **comment** on versions for better tracking.
- **List** and **diff** versions to understand changes.
- **Export** versions and **undo** operations when necessary.

---

## Installation

To install the Easy-Version VCS CLI tool, ensure you have Rust installed and run:

```bash
cargo install artist_vcs_cli
```

---

## Commands Overview

### `save`

**Description**: Save the current state of the tracked files as a new version.

**Syntax**:

```bash
save [message]
```

- **`message`**: (Optional) A descriptive message for the version.

**Example**:

```bash
save "Added new background elements"
```

---

### `load`

**Description**: Load a previously saved version.

**Syntax**:

```bash
load [version_id_or_label]
```

- **`version_id_or_label`**: (Optional) The ID or label of the version to load.

**Examples**:

```bash
load 5
load "final_draft"
```

*If no version is specified, a list of available versions will be displayed for selection.*

---

### `delete`

**Description**: Delete a specified version and all versions dependent on it.

**Syntax**:

```bash
delete [version_id_or_label]
```

- **`version_id_or_label`**: The ID or label of the version to delete.

**Examples**:

```bash
delete 3
delete "old_concept"
```

*Confirmation is required to prevent accidental deletions.*

---

### `split`

**Description**: Create a new project containing all versions up to and including the specified version.

**Syntax**:

```bash
split [version_id_or_label] [new_project_name]
```

- **`version_id_or_label`**: The version to split at.
- **`new_project_name`**: The name of the new project.

**Examples**:

```bash
split 4 "alternate_version"
split "v1.0" "early_stages"
```

---

### `list`

**Description**: List all saved versions with their IDs, labels, and messages.

**Syntax**:

```bash
list
```

**Example Output**:

```
ID  Label        Message
1   initial      "First draft"
2   sketch       "Basic outlines"
3               "Added colors"
4   v1.0         "Ready for review"
```

---

### `label`

**Description**: Assign a custom label to a version for easier reference.

**Syntax**:

```bash
label [version_id] [label_name]
```

- **`version_id`**: The ID of the version to label.
- **`label_name`**: The label to assign.

**Example**:

```bash
label 3 "colored_version"
```

---

### `comment`

**Description**: Add or update a comment for a specific version.

**Syntax**:

```bash
comment [version_id] [message]
```

- **`version_id`**: The ID of the version to comment on.
- **`message`**: The comment message.

**Example**:

```bash
comment 2 "Needs refinement in the upper left corner"
```

---

### `diff`

**Description**: Show differences between two versions.

**Syntax**:

```bash
diff [version1_id_or_label] [version2_id_or_label]
```

**Examples**:

```bash
diff 2 5
diff "sketch" "final"
```

*For binary files, displays metadata differences such as file sizes and timestamps.*

---

### `status`

**Description**: Display the current status, including uncommitted changes and untracked files.

**Syntax**:

```bash
status
```

---

### `help`

**Description**: Provide help information about available commands or a specific command.

**Syntax**:

```bash
help [command]
```

- **`command`**: (Optional) The command to get help for.

**Examples**:

```bash
help
help save
```

---

### `export`

**Description**: Export a specific version to a chosen directory.

**Syntax**:

```bash
export [version_id_or_label] [destination_path]
```

**Examples**:

```bash
export 5 /path/to/destination/
export "v1.0" ~/Desktop/
```

---

### `undo`

**Description**: Revert the last operation if possible.

**Syntax**:

```bash
undo
```

*Use with caution; may not be reversible for all commands.*

---

## Usage Examples

- **Save Work with a Message**:

  ```bash
  save "Completed initial sketch"
  ```

- **List and Load a Version**:

  ```bash
  list
  load "initial_sketch"
  ```

- **Delete a Version**:

  ```bash
  delete 2
  # Confirmation prompt will appear
  ```

- **Split a Project**:

  ```bash
  split "v1.0" "client_presentation"
  ```

- **Assign a Label to a Version**:

  ```bash
  label 5 "final_version"
  ```

- **Add a Comment to a Version**:

  ```bash
  comment 4 "Adjusted brightness levels"
  ```

---

## Notes and Considerations

- **Ease of Use**: Commands are designed with simplicity in mind, using familiar terms.
- **Optional Parameters**: Many commands accept optional parameters for quick actions.
- **Safety Features**: Deletion and undo operations include confirmations to prevent data loss.
- **Version Tracking**: Utilize labels and comments to keep track of versions effectively.
- **Performance**: The tool is optimized for speed and can handle large projects efficiently.
- **Help and Support**: Use the `help` command for guidance on using the CLI tool.

---

## Contact and Support

For support, suggestions, or contributions, please contact:

- **Email**: [vantorrewannes@gmail.com](mailto:vantorrewannes@gmail.com)
- **Website**: 
- **GitHub**: [github.com/VantorreWannes/easyversion](https://github.com/VantorreWannes/easyversion)

---

Thank you for choosing the Easy-Version VCS CLI tool. Happy creating!