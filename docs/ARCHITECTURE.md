# 🏗️ Code Blue Engine — Architecture & Technical Deep Dive

## 🌟 Executive Summary
**Code Blue** is a modular, high-performance gray-box FPS game engine and collaborative multi-user level editor built entirely in **Rust**. It utilizes a pure **Data-Driven Entity-Component-System (ECS)** architecture, high-frequency **120Hz UDP client-server netcode**, parallel rigid-body physics, live embedded scripting with AST hot-reloading, and an off-screen render texture viewport pipeline.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                             CODE BLUE ENGINE                                │
├────────────────────────┬──────────────────────────┬─────────────────────────┤
│     GAMEPLAY LAYER     │       EDITOR LAYER       │     NETWORKING LAYER    │
│  • Tnua KCC Controller │  • Egui Docking Panels   │  • 120Hz UDP Stream     │
│  • Weapon Viewmodel    │  • 3D Render Targets     │  • Action Streaming     │
│  • Ballistics & Recoil │  • Multi-User Gizmos     │  • Lock Arbitration     │
│  • Health & Damage     │  • RON Scene Serde       │  • Snapshot Sync        │
├────────────────────────┴──────────────────────────┴─────────────────────────┤
│                          CORE FOUNDATION & SYSTEMS                          │
│  • Bevy 0.16 ECS Architecture                     • Avian3D Parallel Physics│
│  • Rhai AST Scripting Engine                      • Multi-Threading (rayon) │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 📦 Modular Crate Topology

The codebase is partitioned into distinct crates with strict dependency hierarchies to enforce clean architecture, fast incremental compilation, and modular reuse:

| Crate | Responsibility | Key Dependencies |
| :--- | :--- | :--- |
| **`cb_editor`** | Editor binary entry point; windowing and startup orchestration. | `cb_engine`, `bevy`, `bevy_egui` |
| **`cb_engine`** | Central engine hub: state machine, scheduling, player systems, scripting runtime, editor UI, serialization, and camera pipeline. | `bevy`, `avian3d`, `rhai`, `egui_dock`, `rfd`, `ron` |
| **`cb_game`** | Standalone FPS client binary and gameplay test harness. | `cb_engine`, `cb_movement`, `cb_weapons`, `cb_netcode` |
| **`cb_server`** | Headless dedicated server binary for 120Hz authoritative simulation and multi-user editor relay. | `cb_netcode`, `bevy` |
| **`cb_netcode`** | UDP network protocol, client/server message streams, entity replication, and delta compression. | `lightyear`, `bevy`, `serde` |
| **`cb_weapons`** | First-person viewmodel simulation, dynamic mouse lag/sway, raycast convergence, hitscan ballistics, and recoil patterns. | `bevy`, `avian3d` |
| **`cb_movement`** | Kinematic Character Controller (KCC) FSM, ground detection, jump/sprint/crouch states. | `bevy`, `bevy_tnua`, `bevy_tnua_avian3d` |
| **`cb_physics`** | Avian3D physics integration, collision layers, rigid-body configuration. | `avian3d`, `bevy` |
| **`cb_input`** | Hardware input abstraction (mouse delta, keybindings, input enabling/disabling). | `bevy` |

---

## ⚡ 1. ECS & Scheduling Architecture

Code Blue leverages **Bevy 0.16**'s archetype-based ECS. Systems execute across worker thread pools with automatic dependency resolution:

### Engine State Machine
```rust
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EngineState {
    /// Level design mode. Editor UI active, physics paused, picking & gizmos enabled.
    #[default]
    Edit,
    /// Active gameplay mode. Physics running, FPS controller active, viewmodel enabled.
    Play,
}
```

### Pre-Play Memory Snapshots & Instant Reset
When transitioning from `Edit` to `Play`, the engine executes `on_enter_play_mode`:
1. Traverses all entities carrying `SceneObject`.
2. Serializes their reflectable components into an in-memory RON string (`PlayModeSnapshot`).
3. Tags pre-existing world entities with `KeepOnStop`.
4. Spawns the local player avatar and weapon viewmodel.

On `on_exit_play_mode`:
1. Despawns all entities created during gameplay (projectiles, temporary particles) lacking `KeepOnStop`.
2. Restores the exact RON memory snapshot into the world with 0% state leakage.

---

## 🌐 2. 120Hz Collaborative Multiplayer Netcode

Code Blue uses **Lightyear** over raw UDP with custom action channels for both low-latency gameplay and multi-user editor synchronization:

### Protocol & Action Streaming
```rust
pub enum EditorAction {
    SpawnObject { id: u64, object_type: String, asset_path: Option<String>, transform: Transform },
    DespawnObject { id: u64 },
    UpdateTransform { id: u64, transform: Transform },
    LockObject { id: u64, user_id: u64 },
    UnlockObject { id: u64, user_id: u64 },
    UpdateEditorCamera { user_id: u64, transform: Transform },
    UpdatePlayerTransform { user_id: u64, transform: Transform, pitch: f32 },
    SaveScene { path: String },
    LoadScene { path: String },
    ClearScene,
    StartPlayMode { user_id: u64 },
}
```

### Deterministic User Color Assignment
Every connected user receives a deterministic hue calculated via golden ratio hashing:
$$\text{Hue} = ((\text{user\_id} \times 0.6180339887) \pmod 1) \times 360^\circ$$
This color is seamlessly applied to:
- 3D editor camera frustums.
- Selection wireframes.
- Remote player 3D character avatars and visors.
- Inspector lock badges.

---

## 📜 3. Embedded Scripting Engine (Rhai)

Code Blue embeds the lightweight, safe **Rhai** scripting language with automatic timestamp-based hot-reloading:

* **AST Caching:** Rhai scripts are compiled into Abstract Syntax Trees and cached in `ScriptCache`.
* **Zero-Restart Reloading:** The `reload_modified_scripts` system polls file modification timestamps. When an author saves `.rhai` code in VS Code, the AST is re-compiled in <1ms and swapped atomically into active entities without resetting game state.
* **Engine API Bindings:** Exposes entity transform manipulation (`translate`, `rotate_y`, `set_scale`), math utilities, and logging.

---

## 🎯 4. First-Person Weapon Simulation Pipeline

The weapon mechanics are computed through a multi-stage simulation pipeline in `cb_weapons`:

```
[ Camera World Pose ] ───> Raycast (Center Screen) ───> 3D Target Convergence Point
                                                                 │
[ Mouse Delta (X, Y) ] ──> Dynamic Rotational Drag ────────────> Slerp Orientation
                                                                 │
[ Velocity / FSM ]   ───> Walking Bob & Sway ──────────────────> Local Transform
                                                                 │
[ ShotFiredEvent ]   ───> Recoil Impulse (Z Back, X Up) ────────┘
```

1. **Raycast Aim Alignment:** The weapon calculates the convergence vector from its muzzle to the 3D world crosshair point (`cam_origin + cam_forward * 50.0`).
2. **Dynamic Lag & Sway:** Mouse acceleration applies opposite yaw/pitch inertia (`Quat::slerp`), smoothly dragging behind fast rotations and settling naturally.
3. **Walking Bob & Recoil Kick:** Sinuous bobbing scales with movement speed; weapon firing applies backward displacement and upward muzzle flip that recovers exponentially.

---

## 🖥️ 5. Off-Screen Viewport Rendering Pipeline

The editor achieves smooth panel docking by rendering 3D scenes into off-screen **`ImageRenderTarget`** textures:

* **Editor Viewport Texture:** 1280×720 `Bgra8UnormSrgb` render target bound to `EditorCamera`.
* **Game Viewport Texture:** Independent render target bound to `PlayerCamera`.
* **Dynamic Aspect Ratio Resizing:** Egui panel dimensions dynamically drive the off-screen render target scale to prevent distortion or stretching.
