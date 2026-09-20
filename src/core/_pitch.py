NOTES = ["C", "Cs", "D", "Ds", "E", "F", "Fs", "G", "Gs", "A", "As", "B"]
SUBS  = ["c", "cs", "d", "ds", "e", "f", "fs", "g", "gs", "a", "as", "b"]

OCTAVES = 10

ENUMS_PER_LINE = 1

STEPS_PER_OCTAVE = len(NOTES) * len(SUBS)
COUNT = STEPS_PER_OCTAVE * OCTAVES

A4_INDEX = 4 * STEPS_PER_OCTAVE + NOTES.index("A") * len(SUBS)

SEMITONE_IDENTIFYING_PREFIX = "n"
names = []

for octave in range(OCTAVES):
    for note in NOTES:
        for micro, sub in enumerate(SUBS):
            if micro == 0:
                names.append(f"{note}{octave}")
            else:
                names.append(f"{note}{SEMITONE_IDENTIFYING_PREFIX}{sub}{octave}")

out = []
w = out.append

w("// Do not edit by hand. Regenerate with _pitch.py")
w("")
w("#[allow(dead_code)]")
w("#[repr(u16)]")
w("#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]")
w("pub enum Note {")
for i in range(0, len(names), ENUMS_PER_LINE):
    w("    " + " ".join(f"{n} = {i + j}," for j, n in enumerate(names[i:i + ENUMS_PER_LINE])))
w("}")
w("")
w("impl Note {")
w("    #[allow(dead_code)]")
w("    pub fn frequency_hz(self) -> f64 {")
w(f"        440.0 * 2f64.powf((self as i32 - {A4_INDEX}) as f64 / {STEPS_PER_OCTAVE}.0)")
w("    }")
w("}")
w("")

with open("c_pitch.rs", "w") as f:
    f.write("\n".join(out))
