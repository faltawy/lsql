# Project Overview

LSQL is a command-line tool that queries and manipulates the filesystem using a SQL-like language. The project is organized into four main layers:

1. **Interpreter** - Coordinates CLI input or interactive shell commands and orchestrates the rest of the pipeline.
2. **Parser** - Converts the user's query string into a structured `Query` object defined by the grammar in `src/parser`.
3. **Filtering / FS** - Uses the query structure to traverse the filesystem and filter entries (`src/fs.rs` and `src/filter.rs`).
4. **View** - Formats and displays results to the user using themes and table formatting (`src/display.rs`).

This file acts as a lightweight memory store for the AI agent, summarizing the architecture so it can quickly recall how the project fits together.
