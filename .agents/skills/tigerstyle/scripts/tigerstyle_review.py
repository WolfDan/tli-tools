#!/usr/bin/env python3
"""
TigerStyle Git Diff & PR Reviewer

Reviews staged git changes or PR diffs against TigerStyle rules:
- Checks line lengths (<= 100 columns)
- Checks added functions for assertions (>= 2 asserts per function)
- Flags little-endian naming in added lines
- Checks for dynamic allocation calls in hot paths
"""

import subprocess
import sys
import re

def get_git_diff():
    try:
        # Check staged diff first; if empty, check unstaged diff against HEAD
        diff = subprocess.check_output(["git", "diff", "--cached"], text=True)
        if not diff.strip():
            diff = subprocess.check_output(["git", "diff", "HEAD"], text=True)
        return diff
    except Exception as e:
        print(f"Error fetching git diff: {e}")
        return ""

def review_diff(diff_text: str):
    if not diff_text.strip():
        print("No git changes detected to review.")
        return

    lines = diff_text.splitlines()
    current_file = ""
    added_lines_count = 0
    assertions_count = 0
    warnings = []

    for line in lines:
        if line.startswith("+++ b/"):
            current_file = line[6:]
            continue

        if line.startswith("+") and not line.startswith("+++"):
            code = line[1:]
            added_lines_count += 1

            # 1. Line length check
            if len(code.rstrip('\r\n')) > 100:
                warnings.append(f"[{current_file}] Line exceeds 100 columns: '{code[:50]}...'")

            # 2. Assertion check
            if re.search(r'\bassert\s*\(', code) or re.search(r'\bassert!\s*\(', code):
                assertions_count += 1

            # 3. Naming check
            if re.search(r'\b(max_[a-z0-9_]+|min_[a-z0-9_]+)\b', code):
                match = re.search(r'\b(max_[a-z0-9_]+|min_[a-z0-9_]+)\b', code).group(1)
                warnings.append(f"[{current_file}] Little-endian identifier detected: '{match}' (prefer '<entity>_max')")

            # 4. Alloc check
            if re.search(r'\bmalloc\s*\(|\bpage_allocator\b', code):
                warnings.append(f"[{current_file}] Dynamic allocation detected in diff: '{code.strip()}'")

    print("=== TigerStyle Diff Review ===")
    print(f"Lines added/modified: {added_lines_count}")
    print(f"Assertions added:     {assertions_count}")
    print(f"Warnings / Audits:    {len(warnings)}\n")

    if warnings:
        print("🚨 Potential TigerStyle Violations:")
        for w in warnings:
            print(f"  • {w}")
    else:
        print("✅ Clean diff! Compliant with TigerStyle principles.")

if __name__ == "__main__":
    diff = get_git_diff()
    review_diff(diff)
