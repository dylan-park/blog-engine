# blog-engine

A lightweight backend blog engine for my personal website that automatically converts Markdown files into blog posts. The engine dynamically generates Markdown -> HTML conversions and pagination on page load.

## Features

- **100% Rust** - Built entirely in Rust for performance and reliability
- **Markdown-Based** - Write posts in Markdown with support for basic styling
- **Minimal Static HTML** - Most content is dynamically generated, and applied to basic HTML templates
- **Hot-Reload Content** - New posts appear instantly without server restarts, even template and style changes apply instantly

## Usage

Once the engine is run for the first time, it will generate a posts folder in the same directory as the binary. Simply put Markdown files into this directory and they will instantly be available.

### Post Naming Convention

Posts are expected to have a naming convention similar to: `yyyy-mm-dd-title.md`. This way they are displayed in order from most recent. See [Future Work](#Future-Work) for more details.

### Post Frontmatter

Posts are expected to have a Frontmatter in TOML format. All values are given as strings, and should look like this example:

```
+++
title = "Title"
date = "yyyy-mm-dd"
categories = ["Category"]
+++
```

This data is used while generating the individual post pages, as well as the post cards shown on the home page.

## Future Work

- **In-Memory store of Frontmatter** - Instead of constantly re-reading frontmatter per-access, I want to keep a store of all post's frontmatter data in memory, maintaining it on every update to the posts directory.
    - **Use date from Frontmatter to sort posts** - Initially the naming convention requirement stemmed from the fact that in cases of large collections of posts, the act of scanning each one for its date was much higher complexity than just scanning through the files names one time. However, with the introduction of in-memory frontmatter storage, the job becomes much more manageable in scale.
        - **More Customized URL format** - As a side effect, this will allow the user to have more control over their final post URLs through file naming.
    - **Proper Tagging Engine** - Currently the category system is primarily for show. It would be nice to be able to filter posts by certain categories. Saving myself the complexity of reading the frontmatter for every post on each request makes this possible.

#### Note

>This source code is mostly provided for transparency. While not directly intended for use by others, the project could easily be modified to suit your needs if it interests you. All HTML is controlled through templates, and CSS styling is controlled through static files.