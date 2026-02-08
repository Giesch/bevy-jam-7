# list all available just recipes
list:
    @ just --list --unsorted

# native dev build
dev:
    bevy run

# web dev build
web:
    bevy run web
