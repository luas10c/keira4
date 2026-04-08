# CONTRIBUTING

Thank you for your interest in contributing to this project.

This guide explains how to contribute in a clear, consistent, and organized way. By contributing, you help improve the project and make it more useful for everyone.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How to Contribute](#how-to-contribute)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Improvements](#suggesting-improvements)
- [Development Workflow](#development-workflow)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Guidelines](#pull-request-guidelines)
- [Code Style](#code-style)
- [License](#license)

## Code of Conduct

Please be respectful and professional when participating in this project.

When contributing, keep discussions constructive, clear, and focused on improving the project.

## How to Contribute

You can contribute by:

- Reporting bugs
- Suggesting improvements
- Improving documentation
- Fixing issues
- Adding tests
- Refactoring code
- Implementing new features

Before starting a large change, consider opening an issue first to discuss the idea.

## Reporting Bugs

When reporting a bug, include as much detail as possible.

A good bug report should include:

- A clear and descriptive title
- Steps to reproduce the issue
- The expected behavior
- The actual behavior
- Screenshots, logs, or error messages, if applicable
- Environment details, such as operating system, runtime version, or browser version

Example:

```md
## Description

The button does not trigger the submit action when clicked.

## Steps to Reproduce

1. Open the form page.
2. Fill in all required fields.
3. Click the submit button.

## Expected Behavior

The form should be submitted successfully.

## Actual Behavior

Nothing happens after clicking the button.
```

## Suggesting Improvements

Suggestions are welcome.

When suggesting an improvement, explain:

- What problem the improvement solves
- Why the change would be useful
- How the feature or improvement could work
- Any possible alternatives

## Development Workflow

Follow these steps to contribute code:

1. Fork the repository.
2. Clone your fork:

```bash
git clone https://github.com/luas10c/keira4.git
```

3. Enter the project directory:

```bash
cd keira4
```

4. Create a new branch:

```bash
git checkout -b feature/your-feature-name
```

5. Install dependencies:

```bash
npm install
```

6. Make your changes.
7. Run tests and checks:

```bash
npm test
```

8. Commit your changes.
9. Push your branch:

```bash
git push origin feature/your-feature-name
```

10. Open a Pull Request.

## Commit Guidelines

Use clear and meaningful commit messages.

Prefer the Conventional Commits format:

```txt
type(scope): short description
```

Common commit types:

- `feat`: adds a new feature
- `fix`: fixes a bug
- `docs`: updates documentation
- `style`: changes formatting without affecting behavior
- `refactor`: improves code without changing behavior
- `test`: adds or updates tests
- `chore`: updates tooling, dependencies, or project configuration
- `ci`: changes continuous integration configuration
- `build`: changes build system or external dependencies

Examples:

```bash
feat(auth): add login validation
fix(button): correct disabled state style
docs(readme): update installation instructions
refactor(api): simplify request handler
test(input): add validation tests
chore(deps): update dependencies
```

## Pull Request Guidelines

Before opening a Pull Request, make sure that:

- Your code works as expected
- Tests pass successfully
- The change does not break existing functionality
- The code follows the project style
- The Pull Request has a clear title
- The Pull Request description explains what was changed and why

A good Pull Request description should include:

```md
## Summary

Describe the changes made in this Pull Request.

## Changes

- Added a new feature
- Fixed an issue
- Updated tests

## Checklist

- [ ] I tested my changes
- [ ] I updated the documentation, if needed
- [ ] My code follows the project style
```

## Code Style

Keep the code simple, readable, and consistent.

Recommended practices:

- Use clear and descriptive names
- Avoid unnecessary complexity
- Avoid duplicated code
- Keep functions small and focused
- Write comments only when they add useful context
- Follow the existing formatting and structure of the project

## Tests

When adding or changing functionality, include tests whenever possible.

Before submitting a Pull Request, run the test suite to make sure everything is working correctly:

```bash
npm test
```

If the project has additional checks, run them as well:

```bash
npm run lint
npm run build
```

## Documentation

Update the documentation when your change affects:

- Installation
- Usage
- Configuration
- Public APIs
- Examples
- Project behavior

Good documentation helps other contributors and users understand the project more easily.

## License

By contributing to this project, you agree that your contributions will be licensed under the same license as the project.
