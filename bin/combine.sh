#!/bin/bash

# Change to the src directory

# List of js files to be combined
files=(
    "src/lib.rs" \
    "src/main.rs" \
    "src/parser/ast.rs" \
    "src/parser/lexer.rs" \
    "src/parser/mod.rs" \
    "src/wam/data_structures.rs" \
    "src/wam/mod.rs"
)

# Create or truncate the output file
> ./combined.rs

# Loop through the list of files
for file in "${files[@]}"
do
    # Write the file name as a separator
    echo -e "\n\n---------$file---------\n\n" >> ./combined.rs
    
    # Append the contents of the current file to the output file
    cat "$file" >> ./combined.rs
done
