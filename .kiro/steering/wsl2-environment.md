# WSL2 Environment Standards

## Environment Context

This project is developed in WSL2 (Windows Subsystem for Linux 2) environment on Windows.

## Shell Preferences

- **Primary Shell**: Ubuntu on WSL2
- **Command Execution**: Always prioritize Ubuntu/Linux commands and paths
- **File Paths**: Use Linux-style paths (forward slashes)
- **Terminal**: WSL2 terminal environment

## Development Considerations

- All cargo commands should be executed in the WSL2 Ubuntu environment
- File operations use Linux filesystem semantics
- Network operations work through WSL2 networking layer
- Docker operations (if needed) should use WSL2 Docker integration

## Command Execution

When running commands, expect WSL2 Ubuntu shell behavior and output formatting.