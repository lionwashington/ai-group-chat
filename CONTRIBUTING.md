# Contributing to AI Group Chat

Thanks for your interest in contributing! Here's how to get started.

## Development Setup

```bash
# Prerequisites: Rust (stable), Node.js 18+
git clone https://github.com/lionwashington/ai-group-chat.git
cd ai-group-chat
npm install
npm run tauri dev
```

## Project Structure

- `src-tauri/src/` — Rust backend (commands, database, AI client)
- `src/` — React frontend (components, store, IPC wrappers)
- `docs/` — Architecture and requirements documentation

## Making Changes

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Run tests:
   ```bash
   # Rust tests
   cd src-tauri && cargo test

   # Frontend tests
   npm test
   ```
5. Commit with a clear message
6. Open a Pull Request

## Code Style

- **Rust**: Follow standard `rustfmt` formatting
- **TypeScript/React**: Follow existing patterns in the codebase
- **Two-tier command pattern**: Backend commands use `db_*` functions (testable, pure logic) wrapped by thin `#[tauri::command]` handlers

## What to Work On

- Check [Issues](https://github.com/lionwashington/ai-group-chat/issues) for bugs and feature requests
- See `docs/requirements.md` for the full feature list and status
- See "Future Considerations" in requirements for planned features

## Reporting Bugs

Open an issue with:
- Steps to reproduce
- Expected vs actual behavior
- OS and app version

## Adding a New AI Provider

No code changes needed! AI Group Chat uses the OpenAI-compatible API protocol. Any provider that implements `/chat/completions` works by configuring Base URL + API Key + Model in the Bot Manager.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
