import sys

# Fix perceptual_roughness in serialization.rs
with open('crates/cb_engine/src/editor/serialization.rs', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace("mat.roughness = ed_mat.roughness;", "mat.perceptual_roughness = ed_mat.roughness;")

with open('crates/cb_engine/src/editor/serialization.rs', 'w', encoding='utf-8') as f:
    f.write(content)

# Fix move in ui.rs
with open('crates/cb_engine/src/editor/ui.rs', 'r', encoding='utf-8') as f:
    ui_content = f.read()

ui_content = ui_content.replace(".insert(new_mat);", ".insert(new_mat.clone());")

with open('crates/cb_engine/src/editor/ui.rs', 'w', encoding='utf-8') as f:
    f.write(ui_content)

print("Done")
