---
title: TobySSG Design Guide
publish: true
tags:
    - Index
    - Home
---
# TobySSG Design Guide
TobySSG is both a Note Taking App and a Blog. I want to support various Markdown Flavors.
Such as Wiki Syntax, GitHub Flavored Markdown, and even some Obsidian Plugins like Dataview.

## Minimum Requirements
These are the Bare Minimum Requirements before I publish this blog.

### Generates Html from Markdown files.
From the markdown files with `publish: true` set in the frontmatter,
the html must be generated with the following markdown features implemented.

- Syntax Highlighted Code Blocks
- Latex Support
- Wiki Links
- Dataview

### Has the following utilities.
These are the features that link several generated pages into a single cohesive Digital Garden.

- A Directory
- Reserved Adresses for special pages.
- Search
- Tags
- Light and Dark Mode
- Distinction between Blogs and Notes

```rust
println!("Hello, Everyone!");
```

AAAAAAABBBBBBBBBBBB
