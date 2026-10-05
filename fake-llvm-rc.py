#!/usr/bin/env python3
import sys

if "/?" in sys.argv or "-V" in sys.argv:
    print("OVERVIEW: LLVM Resource Converter\nOPTIONS:\n  /no-preprocess")
    sys.exit(0)

# Search for output file after /fo
out_file = None
for i, arg in enumerate(sys.argv):
    if arg == "/fo" and i + 1 < len(sys.argv):
        out_file = sys.argv[i + 1]
        break
    elif arg.startswith("/fo"):
        out_file = arg[3:]
        break

if out_file:
    # Standard 32-byte Windows empty resource header
    empty_res_header = (
        b"\x00\x00\x00\x00"  # DataSize = 0
        b"\x20\x00\x00\x00"  # HeaderSize = 32
        b"\xff\xff\x00\x00"  # Type = 0xFFFF0000
        b"\xff\xff\x00\x00"  # Name = 0xFFFF0000
        b"\x00\x00\x00\x00"  # DataVersion = 0
        b"\x00\x00"          # MemoryFlags = 0
        b"\x00\x00"          # LanguageId = 0
        b"\x00\x00\x00\x00"  # Version = 0
        b"\x00\x00\x00\x00"  # Characteristics = 0
    )
    with open(out_file, "wb") as f:
        f.write(empty_res_header)

sys.exit(0)
