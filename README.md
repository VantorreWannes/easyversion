# EasyVersion (`ev`)

**Stop copy-pasting your project folders.**

EasyVersion is a tool for **Musicians, 3D Artists, and Game Developers** who want to keep track of their work without the mess.

If your hard drive is full of folders named `Project_Final`, `Project_Final_v2`, and `Project_REAL_FINAL`, this tool is for you.

## Why use EasyVersion?

- **Keep it Clean**: Work in **one** folder. EasyVersion remembers every change you make in the background.
- **Save Space**: Your files are automatically compressed using blazingly fast `zstd` compression. Plus, if you have a huge project and only change one file, EasyVersion doesn't duplicate the whole project—it only stores the new file.
- **Zero Clutter**: Unlike other tools, EasyVersion doesn't put hidden files inside your project folder. All data is safely stored in your computer's central data folder.
- **Safe**: It is built to never accidentally overwrite your current work.

## How it Works

Instead of confusing "branches" or "commits", EasyVersion uses two simple concepts: **Save** and **Split**.

1. **Save**: Takes a snapshot of your folder right now.
2. **Split**: Takes an old snapshot and puts it into a **new folder**.

_Note: "Split" allows you to go back in time or try a new direction without touching your current working folder. It's the safest way to experiment._

## The Desktop App

If the command line isn't your thing, we provide a sleek, modern desktop app!

1. Open the app and choose a folder.
2. See exactly what changed in your timeline (`+ added`, `- removed`, `~ changed` files).
3. Type a quick note and click **Save Version**.
4. Click **Restore to New Folder** on any old version to safely clone your project at that point in time.

## Installation

Download the latest binary or GUI app for your operating system from the [Releases page](https://github.com/wannesvantorre/easyversion/releases).

## Quick Start Guide (CLI)

### 1. Save your work

Run this command inside your project folder to take a snapshot.

```bash
ev save -c "Fixed the lighting on the main character"
```

### 2. See your history

Check what snapshots you have saved for this folder.

```bash
ev list
```

_Output:_

```
Saved versions (2):

1. Initial Draft
2. Fixed the lighting on the main character
```

### 3. Go back in time (Split)

Let's say you liked version 1 better. You want to bring it back, but you don't want to lose what you have now.
**Split** creates a _separate_ folder with the contents of version 1.

```bash
# Creates a new folder called 'MyProject_Old_Idea' containing Version 1
ev split --path ../MyProject_Old_Idea --version 1
```

Now you have two folders: your current one, and the old one. You can open the old one to check settings or copy assets back.

### 4. Free up space

If you're completely done with a project and want to wipe its history to save space, run this inside the folder:

```bash
ev clean
```

This permanently deletes the history for the current folder and automatically sweeps your computer's central storage to erase any large file blobs that are no longer needed by any of your other projects.
