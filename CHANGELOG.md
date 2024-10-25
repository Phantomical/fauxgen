# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.5]
### Fixed
- Nightly utilites are now gated behind an `unstable_nightly` feature.

## [0.1.4]
### Changed
- The `AsyncGenerator` trait is now object-safe.

## [0.1.3]
### Changed
- Async generators now implement `Sync` when the interior future implements `Send`.

## [0.1.2]
### Fixed
- Generators should now be able to implement `Send`.

## [0.1.1]
### Added
- `GeneratorTryStream` wrapper type to more easily create streams of results
  from a generator.

## [0.1.0]
This is the initial release!

### Added
- Generator macros.
- `Generator` and `AsyncGenerator` traits.
- Various wrapper types for using generators as iterators or streams.
