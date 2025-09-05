# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0).

## [Unreleased] 

### Added 
- Added add_writer_file_with_level() and add_writer_stdout_with_level() to specify max level valid only for a specific writer.

### Changed
- In previous versions, the thread name field was replaced by "?" in case the logger was initialized with_thread() but the thread name was not available. Now the logger retrieves and traces the ThreadId.

### Removed 

## [0.0.2] - 2025-09-05
### Added 
- Added add_writer_file_with_level() and add_writer_stdout_with_level() to specify max level valid only for a specific writer.

### Changed
- In previous versions, the thread name field was replaced by "?" in case the logger was initialized with_thread() but the thread name was not available. Now the logger retrieves and traces the ThreadId.

## [0.0.1] - 2025-09-01
### Added
- First release