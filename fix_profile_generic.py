import re

file_path = 'src/cosem/profile_generic.rs'
with open(file_path, 'r') as f:
    content = f.read()

# Pattern: executed_time: 0, followed by optional whitespace/newlines, then NOT followed by use_compact_array_encoding
# We want to insert use_compact_array_encoding: false, after executed_time: 0, if not present.

# Let's verify existing ones first
matches = re.findall(r'executed_time: 0,\s*use_compact_array_encoding: false,', content)
print(f"Found {len(matches)} correct patterns.")

# Regex to find missing ones
# We look for executed_time: 0, followed by closing brace or another field
# But wait, executed_time is usually the last field before my change.
# So it should be followed by '}' or maybe comments then '}'.

# Strategy: Replace all 'executed_time: 0,' with 'executed_time: 0, use_compact_array_encoding: false,'
# then remove duplicates 'use_compact_array_encoding: false, use_compact_array_encoding: false,'

new_content = content.replace('executed_time: 0,', 'executed_time: 0,\n            use_compact_array_encoding: false,')

# Now fix duplicates (where it was already present)
# Pattern: use_compact_array_encoding: false, followed by whitespace/newlines, then use_compact_array_encoding: false,
# This is tricky with regex because of indentation.

# Actually, if I already have it, the replace made it appear twice.
# e.g.
# executed_time: 0,
# use_compact_array_encoding: false,
#
# becomes:
# executed_time: 0,
# use_compact_array_encoding: false,
# use_compact_array_encoding: false,

# So I can just replace the double occurrence with single.
new_content = re.sub(r'(use_compact_array_encoding: false,\s*)+use_compact_array_encoding: false,', 'use_compact_array_encoding: false,', new_content)

# Also handle potential  without comma if exists (I verified it doesn't).

with open(file_path, 'w') as f:
    f.write(new_content)

print("Fixed.")
