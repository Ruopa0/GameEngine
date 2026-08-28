# 📄 Code Blue — CV & Resume Impact Bullet Points

Use these tailored bullet points for your resume, CV, LinkedIn, or portfolio project descriptions. Select the version best suited for your target role.

---

## 🎯 Target Role: Game Engine / Systems Programmer

* **Architected and built Code Blue**, a high-performance 3D FPS game engine and collaborative multi-user level editor from scratch in **Rust** using **Bevy 0.16 ECS**, **Avian3D physics**, and **Lightyear UDP networking**.
* **Engineered a real-time collaborative multi-user 3D editor** with deterministic network lock arbitration, teammate camera frustum streaming, live hierarchy reparenting, and full bidirectional component synchronization via RON reflection.
* **Designed a low-latency 120Hz UDP client-server netcode pipeline**, streaming delta action packets, remote player 3D avatars with kinematic physics colliders, camera pitch orientation, and scene snapshots between dedicated servers and clients.
* **Integrated an embedded Rhai scripting runtime** with an automated file watcher and AST caching mechanism, enabling **<1ms live hot-reloading** of entity gameplay scripts from VS Code without restarting the engine.
* **Implemented an off-screen render target viewport pipeline** and dynamic first-person weapon viewmodel simulation featuring 3D raycast aim convergence, dynamic mouse sway, rotational inertia lag, and recoil physics.
* **Built an authoritative match rules & game mode state machine**, incorporating combat damage, dynamic hitboxes, Target Dummy elimination rules, extraction goal zones, in-game HUDs, and instant play-mode resets.
* **Developed non-destructive RON scene persistence** with in-memory state snapshotting, guaranteeing 0% state leakage when transitioning between live editing and Play mode testing.

---

## 🎯 Target Role: Gameplay & Tools Engineer

* **Developed a modular 3D Level Editor** in Rust with Egui docking panels, native OS file pickers (`rfd`), undo/redo history command stacks, and a searchable component catalog with real-time fuzzy filtering.
* **Created responsive FPS character movement and kinematic control** using `bevy-tnua` and `avian3d`, implementing sprinting, sliding, crouching, jumping, and vaulting state machines alongside capsule colliders and combat health.
* **Built a tactile first-person firearm system** with dynamic aiming lag, procedural walking bobbing, magazine reload cycles, recoil kick recovery, and raycast hitscan ballistics.
* **Implemented interactive in-game HUD overlays and match victory/defeat workflows**, giving level designers and players instant feedback during in-editor playtests.
* **Streamlined developer workflows** by embedding script hot-reloading and one-click IDE integration, reducing gameplay iteration cycles from minutes to sub-second updates.

---

## 🎯 Short Summary (For LinkedIn / Portfolio Card / One-Page Resume)

**Code Blue Engine — Custom 3D FPS Engine & Collaborative Multi-User Editor**  
*Rust • Bevy ECS • Avian3D • Lightyear 120Hz UDP • Rhai Scripting • Egui*
* Engineered a ground-up 3D game engine and collaborative editor in Rust with 120Hz UDP multiplayer replication, bidirectional component synchronization, deterministic multi-user lock arbitration, live Rhai script hot-reloading (<1ms), dynamic weapon viewmodel simulation, capsule physics colliders, win/lose game modes, and non-destructive RON scene serialization. Built in collaboration with Google Antigravity.
