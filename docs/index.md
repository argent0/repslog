# Repslog Documentation

Welcome to the official documentation for `repslog`, a Linux-first command-line workout tracker designed for flexibility across strength training, calisthenics, and cardio.

## Table of Contents

1. [Getting Started](getting-started.md) - Installation and initialization.
2. [Managing Exercises](exercises.md) - How to list, search, and add exercises.
3. [Workouts & Sessions](workouts.md) - Creating and managing your training sessions.
4. [Logging Sets](logging.md) - Detailed guide on logging strength, cluster, and cardio sets.
5. [Statistics & Progress](stats.md) - Tracking your personal records and volume.
6. [Database & Migrations](migrations.md) - Understanding the data storage and schema evolution.
7. [Scripting & Automation](scripting.md) - Using `repslog` in scripts and with pipes.

## Core Philosophy

- **Explicit is better than implicit:** Every critical field that affects the meaning of the data must be explicitly supplied.
- **Scriptable:** Designed to be used in shell pipelines and automated workflows.
- **Flexibility:** Supports everything from simple bodyweight sets to complex rest-pause clusters and detailed cardio metrics.
- **Privacy:** Your data stays on your machine in a local SQLite database.

## Testable Examples

This documentation includes many examples. You can verify that the system is working as expected by running the commands provided in the code blocks.

For example, to check your current version:
```bash
repslog --version
```

To see the help for any command:
```bash
repslog help
```

### Verifying Documentation
You can automatically verify the examples in this documentation by running the included verification script:
```bash
./docs/verify_examples.sh
```
This script runs a series of commands against a temporary database to ensure everything is working as expected.
