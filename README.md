# Cap Desktop Fork

This repository is a Windows desktop fork of [CapSoftware/Cap](https://github.com/CapSoftware/Cap).

It is maintained as a desktop-first workspace focused on Tauri, SolidStart, and the Rust capture and rendering stack required by the Windows application.

## Fork Scope

- desktop application only
- Windows-first local development
- reduced workspace with only the packages and crates still needed by the desktop runtime

## Workspace Layout

- `apps/desktop`
  Tauri desktop shell and SolidStart frontend
- `crates/core`
  Shared Rust foundation crates
- `crates/capture`
  Camera, cursor, and screen capture crates
- `crates/media`
  Media decoding, conversion, and Windows media pipeline support
- `crates/render`
  Recording, rendering, editing, export, and encoder crates
- `crates/dev`
  Desktop development harnesses
- `packages/ui-solid`
  Shared Solid UI components
- `packages/web-api-contract`
  Shared contract types still used by the desktop app
- `packages/tsconfig`
  Shared TypeScript configuration
- `tooling/desktop`
  Windows desktop setup and maintenance scripts

## Project Map

```text
.
├── apps/
│   └── desktop/
├── crates/
│   ├── core/
│   ├── capture/
│   ├── media/
│   ├── render/
│   └── dev/
├── packages/
│   ├── tsconfig/
│   ├── ui-solid/
│   └── web-api-contract/
├── tooling/
│   └── desktop/
├── vendor/
├── Cargo.toml
├── package.json
└── pnpm-workspace.yaml
```

## Development

```powershell
pnpm install
pnpm run desktop:setup
pnpm dev
```

## Workspace Rules

- `pnpm-workspace.yaml` contains only the packages still used by the desktop fork.
- `Cargo.toml` groups the Rust workspace by domain instead of a flat crate list.
- local generated files such as `.codex`, `.pnpm-store`, and desktop timestamp configs stay ignored.

## Fork Note

This repository stays intentionally narrower than the source project. If a file, folder, or dependency does not support the Windows desktop product, it should be simplified, relocated, or removed.
