import random

with open("table_values.rs", "w") as f:
    f.write("[\n")
    for _ in range(8):
        f.write("    [\n")
        for _ in range(8):
            f.write("    [\n")
            for _ in range(256):
                val = random.getrandbits(64)
                f.write(f"        0x{val:016X}u64,\n")
            f.write("    ],\n")
        f.write("    ],\n")
    f.write("]\n")