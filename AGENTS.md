# Repository Guidelines

## Project Structure
- Desktop-only workspace for Windows development.
- Main application: `apps/desktop` with Tauri v2 and SolidStart.
- Shared TypeScript packages:
  - `packages/ui-solid`
  - `packages/web-api-contract`
  - `packages/tsconfig`
- Rust workspace is grouped by domain:
  - `crates/core`
  - `crates/capture`
  - `crates/media`
  - `crates/render`
  - `crates/dev`
- Desktop tooling lives in `tooling/desktop`.

## Build, Run, Verify
- Install dependencies with `pnpm install`.
- Prepare the local desktop toolchain with `pnpm run desktop:setup`.
- Start the desktop app with `pnpm dev`.
- Build the desktop app with `pnpm tauri:build`.
- Run TypeScript checks with `pnpm typecheck`.
- Format JavaScript and TypeScript with `pnpm format`.
- Lint JavaScript and TypeScript with `pnpm lint`.
- Build or test a Rust crate with `cargo build -p <crate>` and `cargo test -p <crate>`.

## Coding Style
- TypeScript uses 2-space indentation and Biome formatting.
- Rust uses `rustfmt` and the workspace clippy configuration from `Cargo.toml`.
- File names use kebab-case.
- Solid components use PascalCase.
- Rust modules use snake_case.
- Rust packages use kebab-case.
- Do not add code comments in generated or edited source files.

## Rust Workspace Rules
- Respect the workspace lints in `Cargo.toml`.
- Never ignore `Result`, `Option`, or other `#[must_use]` values.
- Do not use `dbg!()`.
- Do not drop futures implicitly.
- Prefer `saturating_sub` for time arithmetic.
- Prefer direct iteration over index loops when the index is not needed.
- Prefer `.is_empty()` over length comparisons to zero.
- Prefer direct function references over redundant closures.

## Editing Rules
- Do not edit generated files such as `**/tauri.ts`, `**/queries.ts`, or `apps/desktop/src-tauri/gen/**`.
- Prefer existing package scripts over ad-hoc command chains.
- Keep secrets out of version control and localize configuration in `.env`.
- Treat the repository as a desktop fork first: remove or simplify anything that does not support the Windows desktop product.

## Testing
- Keep tests near the code they validate.
- TypeScript tests use `*.test.ts` or `*.test.tsx` when present.
- Rust tests can live in `src` or `tests`.
- Prefer focused unit tests and lightweight flow checks.

## Commits And PRs
- Use conventional commit prefixes such as `feat:`, `fix:`, `chore:`, `refactor:`, `improve:`, and `docs:`.
- Keep pull requests narrow in scope.
- Update documentation when behavior, structure, or workflows change.

## Formatting
- Run `pnpm format` after TypeScript or JavaScript changes.
- Run `cargo fmt` after Rust changes.
