# LSQL Changelog

## [1.11.0](https://github.com/faltawy/lsql/compare/v1.10.0...v1.11.0) (2025-03-11)

### Features

* Enhance shell experience with Emacs keybindings and validator ([632f3ba](https://github.com/faltawy/lsql/commit/632f3bafcc8b890e58031e6850eb3d11c3547f8d))

### Code Refactoring

* Improve non-recursive directory deletion handling ([469d637](https://github.com/faltawy/lsql/commit/469d637ef1efb8a68fadc7efbd46fe5223058980))
* Remove Files and Directories selection types ([27644c9](https://github.com/faltawy/lsql/commit/27644c904cdf41198bad9a7c42bea399858c6223))

## [1.10.0](https://github.com/faltawy/lsql/compare/v1.9.0...v1.10.0) (2025-03-10)

### Features

* Enhance delete query syntax with FIRST and MANY keywords ([f89914e](https://github.com/faltawy/lsql/commit/f89914e112e4b7e8aa0f1c98f48ea5c8bea3331e))

## [1.9.0](https://github.com/faltawy/lsql/compare/v1.8.0...v1.9.0) (2025-03-10)

### Features

* Enhance 'type' field support in display and filtering ([b0deee3](https://github.com/faltawy/lsql/commit/b0deee37a986935068d98987d66fcaf443ccd29f))

## [1.8.0](https://github.com/faltawy/lsql/compare/v1.7.0...v1.8.0) (2025-03-10)

### Features

* Replace file/directory selection with 'type' field for more SQL-like syntax ([2639623](https://github.com/faltawy/lsql/commit/2639623f1bde1517c158388c60ff2d7d931ff2e5))

## [Unreleased]

### Breaking Changes

* Replace file/directory selection options with a 'type' field for more SQL-like syntax ([commit-hash](https://github.com/faltawy/lsql/commit/))

## [1.7.0](https://github.com/faltawy/lsql/compare/v1.6.0...v1.7.0) (2025-03-10)

### Features

* Add ORDER BY support for sorting file system entries ([82ccc40](https://github.com/faltawy/lsql/commit/82ccc408e3b33bdf7fd530082fddb2f240fd5bac))

### Code Refactoring

* Remove unused imports in cli module ([f77463f](https://github.com/faltawy/lsql/commit/f77463f536b8f4f29dc7b5415f1f8317861dcb2c))

## [1.6.0](https://github.com/faltawy/lsql/compare/v1.5.2...v1.6.0) (2025-03-10)

### Features

* Add DELETE query support with dry run option ([c49002d](https://github.com/faltawy/lsql/commit/c49002da8bdf4facb876e42ff005ce6c71751f99))
* Add LIMIT clause support to LSQL query parser and file listing ([6305a48](https://github.com/faltawy/lsql/commit/6305a48b49dbdb093e7f17104a214124e52e1f1f))
* Enhance DELETE query with recursive deletion and user confirmation ([54dfb0c](https://github.com/faltawy/lsql/commit/54dfb0c483da338d23757b71f896b0ff003922a7))

## [1.5.2](https://github.com/faltawy/lsql/compare/v1.5.1...v1.5.2) (2025-03-10)

### Bug Fixes

* update publish workflow to handle Cargo.lock and add --allow-dirty flag ([badcc54](https://github.com/faltawy/lsql/commit/badcc5484b3ed2cf531ba3f6ba7418fc5102e625))

## [1.5.1](https://github.com/faltawy/lsql/compare/v1.5.0...v1.5.1) (2025-03-10)

### Bug Fixes

* update GitHub workflow to handle git synchronization issues ([945eb49](https://github.com/faltawy/lsql/commit/945eb49479cafc45ea4c7483647541a0d2442610))

## [1.5.0](https://github.com/faltawy/lsql/compare/v1.4.1...v1.5.0) (2025-03-10)

### Features

* add support for cargo install and cargo binstall ([55ab9b4](https://github.com/faltawy/lsql/commit/55ab9b46de2ab0f33decfa397bc199e57e50e1c8))

## [1.4.1](https://github.com/faltawy/lsql/compare/v1.4.0...v1.4.1) (2025-03-10)

### Bug Fixes

* update GitHub release workflow to fix artifact upload issues ([989268f](https://github.com/faltawy/lsql/commit/989268f9751e7e0b068726f558f49235b739118c))

## [1.4.0](https://github.com/faltawy/lsql/compare/v1.3.0...v1.4.0) (2025-03-10)

### Features

* **filter:** Add comprehensive file system filtering module ([85aa263](https://github.com/faltawy/lsql/commit/85aa26395a647c68bfcb4f653f78a02fa246f3da))
* **vscode:** Add debug configuration for LSQL project ([f50a53a](https://github.com/faltawy/lsql/commit/f50a53abc1875ef163851c18e120480225d5b9ff))

### Code Refactoring

* **parser:** Modularize query parsing with separate concern modules ([252fde4](https://github.com/faltawy/lsql/commit/252fde41580974e453a6eea6848afb486ea96769))
* **parser:** Remove unused imports and simplify type references ([5e29d03](https://github.com/faltawy/lsql/commit/5e29d034e009cfecbed8aaf6117dc7a4aeae5d5c))
* **shell:** Improve error handling and shell interaction ([208bdbd](https://github.com/faltawy/lsql/commit/208bdbd327565c585208278177d33eea5819b8c0))

## [1.3.0](https://github.com/faltawy/lsql/compare/v1.2.0...v1.3.0) (2025-03-10)

### Features

* **shell:** Implement interactive shell with enhanced user experience ([79d372e](https://github.com/faltawy/lsql/commit/79d372e015077205405ba564ff3541c1191d5829))

## [1.2.0](https://github.com/faltawy/lsql/compare/v1.1.0...v1.2.0) (2025-03-10)

### Features

* **cli:** Change default log level to 'off' ([8ae8ef6](https://github.com/faltawy/lsql/commit/8ae8ef6afeb7f2c5b49c4a58a4597b4fb38194b5))

## [1.1.0](https://github.com/faltawy/lsql/compare/v1.0.0...v1.1.0) (2025-03-10)

### Features

* **parser:** Enhance query parsing with optional semicolon and improved path matching ([e9e811e](https://github.com/faltawy/lsql/commit/e9e811ed5b56aeae0daf8730898c2bbce4a4604f))

## 1.0.0 (2025-03-10)

### Features

* Add README.md with project description ([8b6e4db](https://github.com/faltawy/lsql/commit/8b6e4db48617446af740faf76375ea1d6bc37d1c))
* Enhance LSQL parser with comprehensive test coverage and operator parsing ([e0a7255](https://github.com/faltawy/lsql/commit/e0a725510356327c4c021eca70afe9654329d686))
* Implement advanced theming system with customizable color and styling options ([ddacd73](https://github.com/faltawy/lsql/commit/ddacd7372b627078ddefc98f30a6d1fed8d288e4))

### Bug Fixes

* trigger workflow with permissions fix ([0066cd9](https://github.com/faltawy/lsql/commit/0066cd92c9195b8d9a9ef785ea9b3354f386c433))

### Code Refactoring

* Simplify string conversions and method calls in display and theme modules ([e7a34fa](https://github.com/faltawy/lsql/commit/e7a34fae55317aa41630c8f27520a591e264dd9e))
* Switch from nom to pest parser and simplify project structure ([32283bb](https://github.com/faltawy/lsql/commit/32283bbc854dff350a008ebb25529ab77f8706ab))

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
