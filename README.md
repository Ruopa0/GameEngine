# Code Blue Engine 🚀

> *"If they can't make their own game engine, they are not programmers. It's like a chef who can only make frozen pizza."*  
> — **Markus "Notch" Persson**

**Code Blue** is a custom, high-performance gray-box FPS game engine and collaborative multi-user level editor built entirely from first principles in **Rust**, powered by **Google's Antigravity**.

---

## 💡 Philosophy & Motivation

Rather than relying on off-the-shelf black boxes, Code Blue was crafted from the ground up to explore the bleeding edge of modern game engine architecture: pure data-driven ECS, native 120Hz UDP multiplayer netcode, live embedded scripting, and real-time multi-user collaborative scene building.

---

## 🚀 Key Features

* **Pure Data-Driven ECS:** Built on [Bevy 0.16](https://bevyengine.org/) for massive CPU parallelization and clean modular architecture.
* **120Hz UDP Multiplayer Netcode:** Powered by `lightyear` and custom delta/snapshot replication for silky-smooth 120Hz client-server multiplayer and play-mode avatar synchronization.
* **Real-Time Collaborative Multi-User 3D Editor (`cb_editor`):**
  * **Independent Viewports & Camera Streaming:** Multiple developers can edit simultaneously with independent 3D cameras and live teammate frustum visualization.
  * **User Color Presence:** Deterministic assigned color palette per user for locks, outlines, camera cones, and inspector badges.
  * **Named Save / Open & Native File Pickers:** Full RON scene persistence with `rfd` native OS file dialogs and in-editor fallbacks.
  * **User-Friendly Simple Mode vs. Developer Mode Inspector:** Clean cards, sliders, color pickers, and raw reflection inspection.
  * **Searchable Component Catalog:** Instant keyword filtering across Physics, Visuals, Scripting, and Identity.
* **Live Script Hot-Reloading:** Embedded [Rhai](https://rhai.rs/) scripting engine with automatic disk timestamp detection. Edit `.rhai` behavior scripts in VS Code and watch them hot-reload in real-time without restarting!
* **Physics & Kinematic Character Controller:** Parallel physics simulation via `avian3d` and responsive FPS movement with `bevy-tnua` (sprinting, jumping, crouching).
* **Tactile FPS Weapon System:**
  * First-person bottom-center viewmodel with center-screen 3D world raycast aiming convergence.
  * Dynamic mouse sway, rotational inertia lag, movement bobbing, and recoil impulses.
* **AI-Accelerated Engineering:** Architected, prototyped, and refined using **Google's Antigravity** as an AI pair-programming copilot.

---

## 📦 Architecture

The project is split into modular crates with clean dependency boundaries:

* `cb_editor`: The collaborative 3D scene editor with docking Egui panels, multi-user visual presence, and scene tools.
* `cb_engine`: Core engine plugins, scheduling, player controller, scene serialization, and Rhai scripting runtime.
* `cb_game`: The standalone FPS gameplay client.
* `cb_server`: Dedicated headless UDP game server.
* `cb_netcode`: High-frequency client-server networking protocol, action relays, and replication (`lightyear`).
* `cb_weapons`: First-person viewmodel, hitscan ballistics, recoil patterns, and weapon simulation.
* `cb_movement`: Movement state machines and Tnua character controllers.
* `cb_physics`: Avian3D rigid-body physics integration layer.
* `cb_input`: Hardware input abstraction and action mapping.

---

---

## 📚 Complete Documentation Index

For in-depth guides, manuals, and technical specifications, explore the **`docs/`** directory:

| Document | Description |
| :--- | :--- |
| **[📘 Editor User & Developer Manual](docs/EDITOR_MANUAL.md)** | Exhaustive manual covering every panel, tool, gizmo, catalog component, multi-user workflow, asset browser, and shortcut. |
| **[🏗️ Engine Architecture Deep Dive](docs/ARCHITECTURE.md)** | Technical breakdown of Bevy ECS scheduling, 120Hz UDP networking protocol, Rhai AST reloader, and off-screen viewport pipeline. |
| **[📜 Rhai Scripting Guide & API Reference](docs/SCRIPTING_GUIDE.md)** | Complete guide to writing behavior scripts, lifecycle hooks (`on_start`, `on_update`), and live hot-reloading with VS Code. |
| **[🎮 Portfolio Technical Case Study](docs/PORTFOLIO_CASE_STUDY.md)** | Polished, in-depth engineering case study highlighting metrics, challenges solved, architecture diagrams, and lessons. |
| **[📄 CV & Resume Impact Bullet Points](docs/CV_RESUME_BULLETS.md)** | High-impact, quantifiable resume bullets tailored for Game Engine, Systems, Gameplay, and Tools Programmer roles. |

---

## 🛠️ Building & Running

Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed.

### 1. Run the Dedicated Server
```bash
cargo run --bin cb_server
```

### 2. Run the Collaborative 3D Editor
```bash
cargo run --bin cb_editor
```

### 3. Run the Standalone Game Client
```bash
cargo run --bin cb_game
```

---

## 🎮 Controls & Shortcuts

### Editor Mode
* **`W` / `E` / `R`:** Translate / Rotate / Scale Gizmos
* **`F`:** Focus Camera on Selected Entity
* **`Delete` / `Backspace`:** Delete Selected Object (with Undo support)
* **`Ctrl + N` / `Ctrl + O` / `Ctrl + S`:** New Scene / Open Scene / Save Scene
* **`Ctrl + Z` / `Ctrl + Y`:** Undo / Redo History Command Stack
* **`~` (Backquote):** Toggle / Focus Diagnostic Console
* **`Alt + Left Drag` / `Alt + Middle Drag`:** Orbit / Pan Camera
* **`Right Drag + WASD`:** Fly-Cam Free Navigation

### Play Mode (FPS Gameplay)
* **`WASD`:** Dynamic movement (Tnua Kinematic Character Controller)
* **`Space`:** Jump
* **`Shift`:** Sprint
* **`Ctrl` / `C`:** Crouch / Slide
* **`Mouse`:** Aim & Look (Dynamic weapon viewmodel sway & lag)
* **`Left Click`:** Fire Weapon (Recoil kick + ballistics)
* **`R`:** Reload Magazine
* **`Esc`:** Release Cursor to Editor / Click Game View to Re-grab

---

## 🤝 Philosophy & Credits
* **Engine Architecture & Programming:** Ruan Prinsloo
* **AI Pair-Programming Partner:** Powered by **Google's Antigravity**
* **Inspiration:** Markus "Notch" Persson's classic philosophy on crafting game engines from first principles.

