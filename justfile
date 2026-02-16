# list all available just recipes
list:
    @ just --list --unsorted

# native dev build
dev:
    bevy run

# web dev build
web:
    bevy run web

# write *.beats.json assets based on automatically extracted timestamps
[unix]
beats:
    ./scripts/extract_beats.py './assets/audio/'

# export aseprite files as one sprite sheet & json atlas in the assets directory
[unix]
sprites:
    cd ./aseprite && \
    aseprite --batch *.aseprite \
        --sheet sprite_sheet.png \
        --data sprite_sheet.atlas.json \
        --filename-format "{title} {frame}" \
        --format json-array && \
    mv sprite_sheet.* ../assets/sprites/
