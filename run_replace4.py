import sys
import re

with open('docs/EDITOR_MANUAL.md', 'r', encoding='utf-8') as f:
    content = f.read()

pattern = re.compile(r'## 13\. Console & Diagnostic Logs.*?## 14\.', re.DOTALL)

replacement = '''## 13. Developer Console

The **Developer Console** is an interactive command-line interface overlay for debugging, runtime configuration, and executing cheat commands. 

* **Toggle Console Hotkey:** Press **`~` (Backquote)** anytime to toggle the Developer Console overlay.
* **Command History:** Use the input field to type commands. The console output history persists during the session.

### Available Commands
- **`help`**: Lists all available commands directly in the console.
- **`clear`**: Clears the console output history buffer.
- **`show_fps <0|1>`**: Toggles a high-visibility FPS overlay in the top-right corner.
- **`fps_max <limit>`**: Sets a strict maximum frame rate ceiling (e.g., `fps_max 144`). Set to `0` for unlimited.
- **`show_physics <0|1>`**: Toggles the Avian3D Physics Debug Renderer, allowing you to visually inspect all rigid bodies, colliders, raycasts, and triggers in the scene.
- **`god_mode <0|1>`**: Grants the player an `ImmortalPlayer` component, preventing death from damage.
- **`heal <amount>`**: Instantly adds the specified amount of health to the player.
- **`noclip <0|1>`**: Toggles player collision layers, allowing you to fly freely through walls and geometry.
- **`spawn_bot`**: Spawns a shootable target dummy exactly 5 meters in front of the player's current location and looking direction.

## 14.'''

content = pattern.sub(replacement, content)

with open('docs/EDITOR_MANUAL.md', 'w', encoding='utf-8') as f:
    f.write(content)
print("Done")
