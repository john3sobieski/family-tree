"""Makes winit (Android backend) report the Back key as unhandled, so the system calls Activity.onBackPressed.

Usage: patch_winit.py <path to winit/src/platform_impl/android/mod.rs>. Fails loudly if winit's code changed.
"""
import re
import sys

path = sys.argv[1]
src = open(path, encoding="utf-8").read()

# The existing arm: volume keys are "unhandled". Add the same for Back right after it.
pattern = re.compile(
    r"(Keycode::VolumeUp \| Keycode::VolumeDown \| Keycode::VolumeMute\s*"
    r"if self\.ignore_volume_keys =>\s*\{\s*input_status = InputStatus::Unhandled\s*\},)"
)
new_src, n = pattern.subn(r"\1\n                    Keycode::Back => input_status = InputStatus::Unhandled,", src, count=1)
if n != 1:
    sys.exit("patch_winit: the volume-keys arm was not found; winit changed, update this script")
open(path, "w", encoding="utf-8").write(new_src)
print("patch_winit: Back key is now unhandled")
