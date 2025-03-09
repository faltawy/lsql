# Commit Message Guidelines

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification for commit messages, which leads to more readable messages that are easy to follow when looking through the project history. This also helps us generate the CHANGELOG automatically.

## Commit Message Format

Each commit message consists of a **header**, a **body**, and a **footer**. The header has a special format that includes a **type**, an optional **scope**, and a **subject**:

```
<type>(<scope>): <subject>
<BLANK LINE>
<body>
<BLANK LINE>
<footer>
```

### Type

Must be one of the following:

* **feat**: A new feature
* **fix**: A bug fix
* **docs**: Documentation only changes
* **style**: Changes that do not affect the meaning of the code (white-space, formatting, etc)
* **refactor**: A code change that neither fixes a bug nor adds a feature
* **perf**: A code change that improves performance
* **test**: Adding missing tests or correcting existing tests
* **chore**: Changes to the build process or auxiliary tools and libraries

### Scope

The scope is optional and could be anything specifying the place of the commit change. For example `parser`, `cli`, `fs`, etc.

### Subject

The subject contains a succinct description of the change:

* Use the imperative, present tense: "change" not "changed" nor "changes"
* Don't capitalize the first letter
* No dot (.) at the end

### Body

The body should include the motivation for the change and contrast this with previous behavior.

### Footer

The footer should contain any information about **Breaking Changes** and also references to issues that this commit addresses.

**Breaking Changes** should start with the word `BREAKING CHANGE:` with a space or two newlines. The rest of the commit message is then used for this.

## Examples

```
feat(parser): add support for <= and >= operators

Added proper handling for less-than-or-equal and greater-than-or-equal operators in SQL parser.
- Fixed grammar rules to prioritize multi-character operators
- Added tests to verify functionality
- Ensured both space and no-space variants work

Fixes #123
```

```
fix(cli): resolve issue with path handling on Windows

Closes #456
```

```
docs: update README with usage examples
```

```
BREAKING CHANGE: change the API for file system operations

The file system operations now require a configuration object instead of
individual parameters.
``` 