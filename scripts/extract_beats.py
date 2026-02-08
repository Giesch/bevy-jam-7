#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = [
#     "essentia",
# ]
# ///

# based on the official tutorial here:
# https://essentia.upf.edu/tutorial_rhythm_beatdetection.html

import essentia.standard as es

import sys
import json

audio_path = sys.argv[1]
json_path = sys.argv[2]

audio = es.MonoLoader(filename=audio_path)()

rhythm_extractor = es.RhythmExtractor2013(method='multifeature')
# https://essentia.upf.edu/reference/std_RhythmExtractor2013.html
bpm, beats, beats_confidence, _estimates, beats_intervals = rhythm_extractor(audio)

json_beats = {
    'bpm': bpm,
    'beats_confidence': beats_confidence,
    'beats': beats.tolist(),
    'beats_intervals': beats_intervals.tolist(),
}

with open(json_path, 'w', encoding='utf-8') as f:
    json.dump(json_beats, f, ensure_ascii=False, indent=4)
