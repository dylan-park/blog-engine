# blog-engine

A lightweight backend blog engine for my personal website that automatically converts Markdown files into blog posts. The engine dynamically generates Markdown -> HTML conversions and pagination on page load.

## Features

- **100% Rust** - Built entirely in Rust for performance and reliability
- **Markdown-Based** - Write posts in Markdown with support for basic styling
- **Minimal Static HTML** - Most content is dynamically generated, and applied to basic HTML templates
- **Hot-Reload Content** - New posts appear instantly without server restarts, even template and style changes apply instantly
- **In-Memory Frontmatter Store** - Post metadata is kept in memory, allowing for quick query access without massive amounts of file reads, and negating the need for a heavy database.

## Usage

Once the engine is run for the first time, it will generate a posts folder in the same directory as the binary. Simply put Markdown files into this directory and they will instantly be available. The file name for the posts will determine their final URL, and should be named with that in mind.

### Post Frontmatter

Post files are expected to have a Frontmatter in TOML format. While none of the values are *required*, missing any of them will have unexpected effects. All values are given as strings, and should look like this example:

```
+++
title = "Title"
date = "yyyy-mm-dd"
categories = ["Category"]
+++
```

- title - The title displayed with the post. *Can be different from the file name.*
- date - A date value stored in string format. Displayed with the post, and used for sorting posts.
- categories - An array of strings, representing different categories the post falls under. *At least one should be provided.* The first item in the array is used as the display category, however, any other categories present will still be used for category queries.

This data is used while generating the individual post pages, as well as the post cards shown on the home page.

### Categories

If you click on any post's category, it will bring up a query for any posts under the same category. Sorting and pagination works the same, the results are just filtered.

### Pagination

Pagination is automatically generated based on the total number of posts available, your current filters, and your current page. All logic is handled automatically.

### Forbidden File Names

While this setup enables you to have the file name (and thus URL) of any post be anything within spec, there are a couple of file names that, if conflicting with reserved paths, will not work properly. These are:

- static
- category
- health

*Currently, posts that share these file names will still render to the blog home page as normal. But once clicked, they will not properly link to their post content. [In the future this might change](#Future-Work).*

## Future Work

- **File Name Blacklist** - If any file is named a term already reserved for other endpoints, it should not render to the blog homepage, or anywhere else.

#### Note

>This source code is mostly provided for transparency. While not directly intended for use by others, the project could easily be modified to suit your needs if it interests you. All HTML is controlled through templates, and CSS styling is controlled through static files.