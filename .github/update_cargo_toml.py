import fileinput

# File to perform replacements in
filename = "Cargo.toml"

# Replacements
replacements = {
    "# [workspace]": "[workspace]",
    '# members': 'members'
}

# Perform replacements
with fileinput.FileInput(filename, inplace=True, backup=".bak") as file:
    for line in file:
        for old, new in replacements.items():
            line = line.replace(old, new)
        print(line, end="")