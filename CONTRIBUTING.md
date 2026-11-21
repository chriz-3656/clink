# Contributing to Clink

Thank you for your interest in contributing to Clink! We welcome contributions from developers, designers, and testers of all experience levels.

---

## Code of Conduct

We are committed to providing a welcoming, inclusive, and harassment-free environment for all participants. Please treat everyone with respect and empathy.

---

## Ways to Contribute

1. **Reporting Bugs**:
   - Check existing [GitHub Issues](https://github.com/chriz-3656/clink/issues) to ensure the bug hasn't already been reported.
   - Open a new issue with clear reproduction steps, device specifications (phone model, Android version, Linux distro, window manager), and logs.
2. **Feature Suggestions**:
   - Open an issue explaining the proposed feature, its use case, and how it aligns with Clink's local-first philosophy.
3. **Submitting Pull Requests**:
   - Fork the repository and create your feature branch: `git checkout -b feature/amazing-feature`.
   - Ensure all automated tests pass:
     ```bash
     PYTHONPATH=linux/src:linux/tests python3 -m pytest -v linux/tests
     cd android && ./gradlew testDebugUnitTest
     ```
   - Commit your changes with clear, descriptive commit messages.
   - Open a Pull Request targeting the `main` branch.

---

## Coding Standards

- **Python (Linux Companion)**: Follow PEP 8 guidelines. Use type annotations (`typing`) and ensure clean separation between Cairo rendering, X11 input, and network dispatch.
- **Kotlin (Android)**: Follow standard Android Kotlin coding conventions and Jetpack Compose best practices. Maintain state management in ViewModels and avoid side-effects in composables.
- **Documentation**: Keep `README.md`, `CHANGELOG.md`, and relevant docs in `docs/` updated whenever modifying features or network behavior.
