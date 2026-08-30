import sys
import re

with open('crates/cb_engine/src/editor/serialization.rs', 'r', encoding='utf-8') as f:
    content = f.read()

content = content.replace("}\n            \"weapon_chest\" =>", "},\n            \"weapon_chest\" =>")

with open('crates/cb_engine/src/editor/serialization.rs', 'w', encoding='utf-8') as f:
    f.write(content)
print("Done")
