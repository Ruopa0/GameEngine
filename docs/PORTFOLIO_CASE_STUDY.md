# 🎮 Case Study: Code Blue — High-Performance 3D FPS Engine & Collaborative Multi-User Editor in Rust

> *"If they can't make their own game engine, they are not programmers. It's like a chef who can only make frozen pizza."*  
> — **Markus "Notch" Persson**

**Role:** Engine & Systems Programmer  
**Tech Stack:** Rust, Bevy 0.16 ECS, Avian3D Physics, Lightyear UDP (120Hz), Rhai Embedded Scripting, Egui, Rayon, RFD, RON  
**Development Partner:** Built in collaboration with **Google Antigravity**  
**Repository:** [GitHub: Code Blue Engine](https://github.com/ruan-prinsloo/code_blue)

---

## 📌 Project Overview
**Code Blue** is a bespoke, ground-up FPS game engine and real-time collaborative 3D level editor engineered in pure **Rust**. Born from the ambition to master low-level engine architecture without relying on black-box commercial engines ("frozen pizza"), Code Blue solves the hardest challenges in modern interactive software: **low-latency 120Hz client-server multiplayer**, **live bidirectional multi-user scene & component synchronization**, **embedded scripting with sub-millisecond hot-reloading**, **tactile first-person weapon simulation with physics hitboxes**, and **real-time competitive game mode state machines**.

---

## 🚀 Key Engineering Highlights

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           KEY METRICS & HIGHLIGHTS                          │
├──────────────────────────────────────┬──────────────────────────────────────┤
│ ⚡ 120Hz Authoritative UDP Netcode   │ 🦀 100% Memory-Safe Pure Rust        │
│ 🌐 Bidirectional Component Sync      │ 📜 <1ms Rhai Script Hot-Reloading    │
│ 🎯 Raycast Aim & Weapon Inertia Lag  │ 🧱 Avian3D Capsule Colliders & Combat│
│ 🗂️ Non-Destructive RON Serialization │ 🏆 Win/Lose Game Modes & Custom HUD  │
└──────────────────────────────────────┴──────────────────────────────────────┘
```

### 1. Collaborative Multi-User Level Editor (`cb_editor`)
* **Independent Camera Frustums & Multi-User Presence:** Multiple level designers can inspect and edit the live 3D world simultaneously with independent free-cam viewpoints, live teammate frustum rays, and 3D avatar presence.
* **Deterministic User Color Assignment:** Implemented golden ratio hue hashing to dynamically color-code user selection bounding boxes, 3D frustum cones, and lock badges across clients.
* **Lock Arbitration & Hierarchy Concurrency:** Designed a network lock and reparenting protocol (`LockObject`, `UnlockObject`, `ReparentObject`) to guarantee deterministic scene graph manipulation without race conditions.
* **Dual-Tier Live Inspector:** Created an accessible **Simple Mode** (visual cards, color pickers, physics presets) alongside a **Developer Mode** (full ECS reflection tree) that streams all property edits across peers in real-time.

### 2. Low-Latency 120Hz UDP Networking Architecture (`cb_netcode`)
* Built on top of **Lightyear** with custom delta action streaming over raw UDP.
* Replicates character transforms, pitch rotations, player spawning, weapon firing events, and scene states across dedicated servers (`cb_server`) and clients (`cb_editor`, `cb_game`).
* **Bidirectional Component Synchronization:** Serializes all modified components into RON strings and synchronizes physics bodies, PBR materials, lights, and scripts across connected peers with 0% desync.
* Added multiplayer play-mode invitations, enabling developers to jump into live multiplayer deathmatch sessions directly from the editor.

### 3. Live Embedded Scripting with Zero-Restart Hot-Reloading (`cb_engine`)
* Integrated **Rhai** as an embedded scripting language without garbage-collector spikes.
* Built an automated file watcher and AST cache that polls file modification timestamps and hot-reloads behavior scripts in **<1 millisecond** while preserving active scene state.
* Integrated seamless one-click VS Code launching directly from the in-engine Inspector.

### 4. Tactile FPS Weapon Simulation & Viewmodel Dynamics (`cb_weapons`)
* Implemented a bottom-center first-person weapon viewmodel with dynamic 3D world raycast convergence on the crosshair.
* Designed rotational inertia lag (`Quat::slerp`) and mouse sway, delivering heavy, tactile firearm mechanics.
* Integrated walking bobbing curves, recoil impulse recovery, and hitscan ballistic tracers.

### 5. Avian3D Colliders, Combat Health & Win/Lose Game Modes (`cb_engine::gamemode`)
* **Physics Colliders & Hitboxes:** Configured local and remote players with Avian3D capsule colliders (`Collider::capsule(0.35, 1.0)`), dynamic/kinematic rigid bodies, and health tracking.
* **Real-Time Match Rules:**
  * **Win Conditions (🏆 VICTORY):** Eliminating all Target Dummies in the scene or reaching the designated `GoalZone` extraction volume.
  * **Lose Conditions (💀 DEFEAT):** Player health falling to `0 HP` or falling into the level void (`y < -25.0`).
* **In-Game Tactical HUD:** Features a dynamic health bar, ammo/reload counter, objective timer, crosshair reticle, and celebratory victory/defeat modal flows with instant respawn support.

### 6. Deterministic Memory Snapshots & Play Mode Reset
* Developed an in-memory RON serializer that captures a snapshot of all active `SceneObject` entities upon pressing **▶ Play**.
* Restores the exact pre-play scene state upon pressing **⏹ Stop**, isolating gameplay entities (bullets, debris) from level data with 0% state leakage.

---

## 🛠️ Architecture Deep Dive

```
crates/
├── cb_editor   # Dockable Egui editor entry point, multi-user visual presence, game HUD
├── cb_engine   # Core engine loop, player controller, scene serializer, Rhai runtime, gamemode
├── cb_game     # Standalone FPS client test harness
├── cb_server   # Headless 120Hz authoritative dedicated server
├── cb_netcode  # UDP action streaming, packet layout, multiplayer replication, component sync
├── cb_weapons  # Viewmodel simulation, aim convergence, recoil patterns, health, ballistics
├── cb_movement # Kinematic character controller FSM (Tnua)
├── cb_physics  # Avian3D parallel rigid-body physics integration
└── cb_input    # Hardware input abstraction & action mapping
```

---

## 💡 Lessons & Reflection
Building Code Blue demonstrated that crafting a custom game engine in Rust provides unmatched memory safety, blazing multithreaded performance, and deep architectural freedom. Partnering with **Google Antigravity** as an AI pair-programmer accelerated the design, prototyping, and verification cycles, turning complex multi-crate systems from concept into reality in record time.

