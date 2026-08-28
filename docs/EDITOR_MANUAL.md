# 📘 Code Blue Editor — Complete User & Developer Manual

Welcome to the **Code Blue Collaborative 3D Scene Editor** (`cb_editor`). This manual provides a comprehensive, step-by-step guide to all editor capabilities, interface panels, tools, hotkeys, collaborative multiplayer workflows, scripting, and scene management.

---

## 📑 Table of Contents
1. [Editor Overview & Docking Layout](#1-editor-overview--docking-layout)
2. [Top Menu Bar & Toolbar](#2-top-menu-bar--toolbar)
3. [3D Viewport & Camera Navigation](#3-3d-viewport--camera-navigation)
4. [3D Transform Gizmos & Selection](#4-3d-transform-gizmos--selection)
5. [Scene Hierarchy Panel](#5-scene-hierarchy-panel)
6. [Inspector Panel (Simple vs. Developer Mode)](#6-inspector-panel-simple-vs-developer-mode)
7. [Component Catalog & Adding Components](#7-component-catalog--adding-components)
8. [Asset Browser & Drag-and-Drop Prefabs](#8-asset-browser--drag-and-drop-prefabs)
9. [Live Rhai Script Hot-Reloading & VS Code Integration](#9-live-rhai-script-hot-reloading--vs-code-integration)
10. [Real-Time Multi-User Collaboration](#10-real-time-multi-user-collaboration)
11. [Scene Management & Persistence (Save / Open / New)](#11-scene-management--persistence-save--open--new)
12. [Play Mode Testing & FPS Gameplay](#12-play-mode-testing--fps-gameplay)
13. [Console & Diagnostic Logs](#13-console--diagnostic-logs)
14. [Keyboard Shortcuts Quick Reference](#14-keyboard-shortcuts-quick-reference)

---

## 1. Editor Overview & Docking Layout

Code Blue features a modular, dockable user interface powered by `egui_dock`. You can drag tab headers to rearrange panels, split views horizontally or vertically, or undock windows to tailor your workspace.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ 📁 File  📦 Spawn  📖 Help  │  Move(W) Rotate(E) Scale(R)  │  ▶ Play  📄 Scene  ● Online │
├──────────────┬──────────────────────────────────────────────┬───────────────┤
│  Hierarchy   │  Viewport / Game View                        │  Inspector    │
│              │                                              │               │
│ • Root       │  [ 3D Interactive Scene View ]               │ • Simple Mode │
│   ├─ Light   │                                              │ • Dev Mode    │
│   └─ Cube    │  Teammate Frustums & Live Gizmos             │ • Properties  │
│              │                                              │ • Add Comp    │
├──────────────┴──────────────────────────────────────────────┴───────────────┤
│  Console  |  Assets (Prefabs & File Browser)                                │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Top Menu Bar & Toolbar

The top bar gives one-click access to global engine operations:

### 📁 `File` Menu
* **📄 New Scene (`Ctrl + N`):** Clears the current world and resets the active scene to a clean default state.
* **📂 Open Scene... (`Ctrl + O`):** Opens the native OS file picker (`rfd`) to load any `.ron` or `.scn` scene file.
* **💾 Save Scene (`Ctrl + S`):** Saves current scene changes directly to the active file path.
* **💾 Save Scene As... (`Ctrl + Shift + S`):** Prompts for a new file name to save the current world state as a new level.

### 📦 `Spawn` Menu
* **🗂 Empty Entity:** Instantiates a clean root `Transform` node.
* **📦 Spawn Cube:** Spawns a physical 1×1×1 textured gray-box cube with rigid body physics.

### 📖 `Help` Menu
* **📖 Documentation & Quick Start:** Opens an in-editor cheatsheet with shortcuts, controls, and workflows.
* **ℹ️ About Code Blue...:** Displays the engine credits, architecture summary, Google Antigravity attribution, and Notch's philosophical quote.

### 🛠️ Transform Gizmo Selector
* **Move (W):** Switches 3D gizmo to translation mode.
* **Rotate (E):** Switches 3D gizmo to rotational arc mode.
* **Scale (R):** Switches 3D gizmo to uniform / axis scale mode.

### ▶ Play / ⏹ Stop Mode
* **▶ Play:** Prompts to save the scene, snapshots world state into memory, spawns the local player avatar, and switches focus to the **Game View** with instant mouse grab.
* **⏹ Stop:** Instantly restores the exact pre-play snapshot, cleans up temporary gameplay entities (bullets, debris), and switches back to the 3D **Viewport**.

### 🏷️ Status Badges
* **📄 Scene Badge:** Displays the currently open scene file name (e.g., `📄 Scene: level.ron`).
* **● User ID & Network Badge:** Displays real-time server connectivity and your assigned teammate color (e.g., `● Online | User #4812`).

---

## 3. 3D Viewport & Camera Navigation

The **Viewport** tab renders the scene into an off-screen render texture, supporting fluid editor camera navigation:

* **Orbit Camera (`Alt + Left Mouse Drag`):** Orbits smoothly around the current focal target.
* **Pan Camera (`Alt + Middle Mouse Drag` or `Shift + Right Mouse Drag`):** Panning across the horizontal and vertical screen axes.
* **Zoom (`Mouse Scroll` or `Alt + Right Mouse Drag`):** Zooms towards and away from the focus point.
* **Fly-Cam (`Right Mouse Drag + WASD`):**
  * `Right Mouse Drag`: Look around.
  * `W / S`: Move forward / backward.
  * `A / D`: Strafe left / right.
  * `Q / E`: Ascend / descend.
  * `Shift`: Boost camera speed (2.5×).
* **Focus on Object (`F`):** Smoothly snaps and centers the editor camera on the currently selected entity.

---

## 4. 3D Transform Gizmos & Selection

### 🎯 Picking & Object Selection
* **Left-Click in Viewport:** Raycasts against 3D object bounding boxes and selects the clicked entity.
* **Network Lock Arbitration:** Selecting an object automatically broadcasts an `EditorActionRequest::LockObject` to the server. Other connected peers will see a bounding box tinted with your user color and cannot accidentally overwrite your transform!
* **Deselection:** Clicking on empty space deselects and unlocks the object.

### 📐 Transform Manipulation
* **Translate Handle (`W`):** Red (X), Green (Y), Blue (Z) arrows for axis-constrained movement.
* **Rotate Handle (`E`):** Concentric Euler rotation rings for pitch, yaw, and roll manipulation.
* **Scale Handle (`R`):** Bounding cube handles for precise sizing.
* **Live Network Sync:** Transformations stream continuously at 120Hz to all connected peers.

---

## 5. Scene Hierarchy Panel

The **Hierarchy** panel displays a live tree view of every entity currently in the level:

* **Entity Labels:** Shows the object type (e.g., `Cube`, `Light`, `SpawnPoint`), human-readable name, network ID badge (`[🌐 Net #1234]`), and lock owner (`[🔒 You]` or teammate color badge).
* **Search & Filter:** Type to quickly find entities by name or type.
* **Drag-and-Drop Reparenting:** Drag any entity onto another entity to create a parent-child relationship. Drag onto the root area to unparent.
* **Drag-and-Drop Script Attachment:** Drag `.rhai` files from the Asset Browser directly onto any entity in the Hierarchy to attach a behavior script!
* **Quick Delete (`Delete` / `Backspace`):** Despawns the selected entity with undo history recording.

---

## 6. Inspector Panel (Simple vs. Developer Mode)

The **Inspector** panel provides property editing tailored to all experience levels:

### 🌟 Simple Mode (Friendly / Accessible)
* **Transform Cards:** Visual sliders and numeric inputs with reset buttons.
* **Physics Card:** Easy radio selectors for Body Type (*Dynamic, Static, Kinematic*) and gravity slider.
* **Visuals & Color Picker:** Interactive color palette picker and metallic/roughness sliders.
* **Scripting Card:** Displays the attached `.rhai` script with an **"✏️ Open in VS Code"** button and live reload indicator.

### 🛠️ Developer Mode (Deep Systems Inspection)
* **Full ECS Reflection Tree:** Inspects every low-level Bevy ECS component attached to the entity.
* **Raw Component Editing:** Modify raw values, struct fields, and internal metadata.
* **Quick Remove:** One-click removal of any attached component.

---

## 7. Component Catalog & Adding Components

Click **"➕ Add Component"** at the bottom of the Inspector to open the **Searchable Component Catalog**:

* **Live Fuzzy Search:** Type names, keywords, or descriptions (e.g., `"physics"`, `"light"`, `"rhai"`, `"box"`, `"mass"`).
* **Organized Categories:**
  * **⚡ Physics & Motion:** `RigidBody`, `GravityScale`, `Collider (Box)`, `Collider (Sphere)`, `Collider (Capsule)`, `LockedAxes`.
  * **🎨 Visuals & Rendering:** `MeshMaterial3d (PBR Material)`, `PointLight`.
  * **📜 Gameplay & Scripting:** `RhaiScript (Behavior Script)`.
  * **🏷️ General & Identity:** `Entity Name`, `Transform`.
* **Instant Application:** Click **"➕ Add"** to immediately attach the component, register reflection, and sync across the network.

---

## 8. Asset Browser & Drag-and-Drop Prefabs

The bottom **Assets** tab houses the built-in Prefabs and Project File Browser:

### 📦 Quick Prefab Spawners
* **🗂 Empty Node:** Spawns a clean transform anchor.
* **🎯 Target Dummy (Cube):** Spawns a physics-enabled target cube with 50 HP and `TargetDummy` component.
* **💡 Point Light:** Spawns an omnidirectional light source with gizmo visualization.
* **🚩 Spawn Point:** Spawns a multiplayer player start position.
* **🏁 Goal Zone:** Spawns an extraction goal volume with cylinder sensor collider.

### 📁 Project Asset Browser
* **Folder Navigation:** Click folders to navigate; click **"🔙 Back"** to traverse up.
* **Drag-and-Drop 3D Models:** Drag `.gltf` or `.glb` files directly into the 3D Viewport to spawn the model into the scene.
* **One-Click Script Editing:** Click any `.rhai` script to open it in **VS Code** (or default editor).
* **Model Import Dialog:** Import external 3D meshes with relative asset path resolution.

---

## 9. Live Rhai Script Hot-Reloading & VS Code Integration

Code Blue includes an embedded, high-performance **Rhai AST Scripting Runtime**:

```
┌─────────────────┐       Auto Disk Watcher       ┌───────────────────┐
│   VS Code /     │ ────────────────────────────> │  Code Blue Engine │
│   Text Editor   │   File Timestamp Change (<1s) │  Hot-Reloads AST  │
│  (edit .rhai)   │                               │  Preserves State  │
└─────────────────┘                               └───────────────────┘
```

### Writing a Script
Scripts attached to entities can define lifecycle functions:
```rust
// assets/scripts/spin.rhai
fn on_start() {
    print("Entity script initialized!");
}

fn on_update() {
    // Access and mutate entity transform smoothly
    rotate_y(0.02);
}
```

### Hot-Reload Workflow:
1. Select any entity in the Hierarchy.
2. In the Inspector, attach a `.rhai` script or select an existing one.
3. Click **"✏️ Open in VS Code"**.
4. Edit rotation speed, variables, or behavior logic, then press **`Ctrl + S`** in VS Code.
5. **Code Blue instantly re-compiles and hot-reloads the script in real-time** without dropping frames or restarting!

---

## 10. Real-Time Multi-User Collaboration

Code Blue's editor was engineered from the ground up for simultaneous multi-user level editing:

* **Independent 3D Cameras:** Every connected user navigates their own camera view freely.
* **Live Teammate Frustums:** Teammates appear in your 3D viewport as stylized camera cones tinted with their unique user color.
* **Color-Coded Selection Wireframes:** When a teammate selects an entity, a slim `+ 0.1` wireframe bounding box in their color appears, indicating active editing.
* **Lock Arbitration:** Prevents conflicting simultaneous edits while allowing teammates to build adjacent structures together in real-time.
* **Bidirectional Component Sync:** Physics properties, PBR material colors, lights, and scripts stream live to all teammates with 0% desync.
* **Hierarchy Reparenting Sync:** Dragging and dropping entities in the Hierarchy tree syncs the scene graph across all connected peers.
* **Multiplayer Play-Testing Invites:** When one developer clicks **▶ Play**, all connected peers receive an instant prompt: *"Another user has started Play Mode. Would you like to join?"*.

---

## 11. Scene Management & Persistence (Save / Open / New)

Scenes are serialized into human-readable, version-control-friendly **RON (Rusty Object Notation)**:

* **Native OS File Dialogs:** Seamless file browsing via `rfd` with automatic fallback to in-editor browser.
* **Undo / Redo History (`Ctrl + Z` / `Ctrl + Y`):** Complete command stack tracking spawns, despawns, transform movements, and property changes.
* **Automatic Active Scene Tracking:** The editor remembers the currently open scene file for rapid one-key saves (`Ctrl + S`).

---

## 12. Play Mode Testing, Combat & Game Modes

Test your level mechanics instantly with native FPS controls and authoritative game rules:

* **Auto-Focus Game View:** Pressing **▶ Play** switches the dock tab to **Game View** and immediately locks the OS cursor.
  * `WASD`: Dynamic movement with Tnua Kinematic Character Controller.
  * `Space`: Jump.
  * `Shift`: Sprint.
  * `Ctrl` / `C`: Crouch / Slide.
  * `Mouse`: Look & Aim.
* **Tactile Weapon Viewmodel:**
  * Bottom-center first-person firearm viewmodel.
  * Screen-center 3D world raycast aim convergence.
  * Dynamic mouse lag, sway, and walking bobbing.
  * Left-Click firing with recoil kick and yellow tracer visualization.
  * `R`: Magazine reload.
* **Avian3D Player Colliders & Hitboxes:** Local and remote players feature capsule colliders (`Collider::capsule(0.35, 1.0)`), dynamic rigid bodies, and combat health tracking.
* **Dynamic In-Game HUD:**
  * **Center Reticle Crosshair (`+`)**: Clean aiming reticle.
  * **Health Bar (Bottom-Left)**: Real-time color-coded health (`100 / 100 HP`).
  * **Ammo Counter (Bottom-Right)**: Current magazine and reserve ammo (`15 / 60`).
  * **Objective & Timer HUD (Top-Center)**: Remaining target count and match clock (`🎯 Targets: 3 | ⏱️ 01:45`).
* **Win & Lose Conditions:**
  * **🏆 Victory:** Eliminating all target dummies or stepping onto a `GoalZone` extraction area triggers the Victory modal.
  * **💀 Defeat:** Dropping to `0 HP` or falling below the level void threshold (`y < -25.0`) triggers Defeat.
  * **Respawn & Reset:** Press **"🔄 Respawn / Restart"** in the overlay modal to reset health and teleport back to the spawn point.
* **Multiplayer Play Sync:** Connected peers in Play Mode appear as 3D avatars with synchronized pitch aiming and unique assigned user colors.
* **Release Cursor:** Press `Escape` to unlock the cursor; click anywhere on the Game View to jump back into gameplay.

---

## 13. Console & Diagnostic Logs

* **Bottom Dock Tab:** Displays real-time engine events, asset loading notifications, netcode connection drops/reconnects, and script print statements.
* **Toggle Console Hotkey:** Press **`~` (Backquote)** anytime to immediately focus the Console tab.

---

## 14. Keyboard Shortcuts Quick Reference

| Shortcut | Action | Scope |
| :--- | :--- | :--- |
| **`W`** | Switch to Translate Gizmo | Editor Viewport |
| **`E`** | Switch to Rotate Gizmo | Editor Viewport |
| **`R`** | Switch to Scale Gizmo | Editor Viewport |
| **`F`** | Focus Camera on Selected Entity | Editor Viewport |
| **`Delete` / `Backspace`** | Delete Selected Object | Editor |
| **`Ctrl + N`** | New Scene | Global |
| **`Ctrl + O`** | Open Scene File | Global |
| **`Ctrl + S`** | Quick Save Scene | Global |
| **`Ctrl + Shift + S`** | Save Scene As... | Global |
| **`Ctrl + Z`** | Undo Last Action | Editor |
| **`Ctrl + Y` / `Ctrl + Shift + Z`** | Redo Action | Editor |
| **`~` (Backquote)** | Toggle / Focus Console | Global |
| **`Alt + Left Drag`** | Orbit Camera | Viewport |
| **`Alt + Middle Drag`** | Pan Camera | Viewport |
| **`Right Drag + WASD`** | Fly-Cam Movement | Viewport |
| **`Escape`** | Release Cursor from Game View | Play Mode |

---
*Code Blue Engine — Crafted in Rust • Powered by Google Antigravity*
