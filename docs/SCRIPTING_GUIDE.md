# 📜 Code Blue Scripting Guide & Rhai API Reference

Code Blue features an embedded, zero-cost **Rhai** scripting environment. Rhai provides high-performance, memory-safe scripting without garbage collection pauses, tailored for live gameplay logic and rapid prototyping.

---

## ⚡ Quick Start: Your First Script

1. Create a `.rhai` file inside `assets/scripts/` (e.g. `assets/scripts/spin.rhai`).
2. Add your logic:
```rust
// assets/scripts/spin.rhai

fn on_start() {
    print("🚀 Spin script initialized!");
}

fn on_update() {
    // Continuously rotate around the Y axis
    rotate_y(0.02);
}
```
3. Open the **Code Blue Editor**, select an entity in the Hierarchy or 3D Viewport.
4. Drag `spin.rhai` from the **Assets** panel onto the entity, or click **"➕ Add Component" -> "📜 RhaiScript"**.
5. Press **▶ Play** to see it spin in real-time!

---

## 🔄 Live Hot-Reloading Workflow

Code Blue monitors file modification timestamps on disk. You don't need to rebuild or restart the editor to modify script behaviors:

1. In the Inspector panel for an entity with a script, click **"✏️ Open in VS Code"**.
2. Change values (e.g. change `rotate_y(0.02)` to `rotate_y(0.1)`).
3. Save the file in VS Code (`Ctrl + S`).
4. **Code Blue automatically re-compiles the script in <1ms and updates the entity live!**

---

## 📚 Standard Lifecycle Functions

| Function | Execution Point | Use Case |
| :--- | :--- | :--- |
| `fn on_start()` | Called once when Play mode starts or when the script is attached. | Initialization, state setup, logging. |
| `fn on_update()` | Called every frame during active Play mode. | Movement, continuous animation, physics interaction. |

---

## 🛠️ Built-In Engine API Reference

### Transform Manipulation
* `translate(dx, dy, dz)`: Moves the entity relative to its current local position.
* `rotate_x(radians)`: Rotates the entity around its local X axis (pitch).
* `rotate_y(radians)`: Rotates the entity around its local Y axis (yaw).
* `rotate_z(radians)`: Rotates the entity around its local Z axis (roll).
* `set_scale(x, y, z)`: Sets the scale dimensions of the entity.

### Utilities & Logging
* `print(msg)`: Prints diagnostic messages to the in-engine **Console** dock tab and standard terminal output.

---

## 💡 Practical Examples

### 1. Floating & Bobbing Pickups
```rust
// assets/scripts/pickup_bob.rhai
let time = 0.0;

fn on_update() {
    time += 0.03;
    let offset_y = sin(time) * 0.005;
    translate(0.0, offset_y, 0.0);
    rotate_y(0.03);
}
```

### 2. Moving Hazard Platform
```rust
// assets/scripts/patrol_platform.rhai
let direction = 1.0;
let distance = 0.0;

fn on_update() {
    let speed = 0.05 * direction;
    translate(speed, 0.0, 0.0);
    distance += speed;
    
    if distance > 5.0 {
        direction = -1.0;
    } else if distance < -5.0 {
        direction = 1.0;
    }
}
```
