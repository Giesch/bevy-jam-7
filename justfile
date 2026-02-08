# list all available just recipes
list:
    @ just --list --unsorted

# native dev build
dev:
    bevy run

# web dev build
web:
    bevy run web

# write beats.json based on automatically extracted timestamps
beats:
    ./scripts/extract_beats.py \
      './assets/audio/03_Scherzo_Allegro_vivace.flac' \
      './assets/data/beats.json'
