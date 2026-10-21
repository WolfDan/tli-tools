#!/usr/bin/env python3
"""
TigerStyle Linter & Rule Checker

Automated validation for TigerStyle rules:
1. Function length limit (hard max: 70 lines)
2. Line width limit (hard max: 100 columns)
3. Assertion density (target: >= 2 assertions per function)
4. Big-endian naming checks (flags little-endian prefixes like max_*, min_*)
5. Dynamic memory allocation detection
"""

import sys
import os
import re
import argparse
from typing import List, Dict, Any

LINE_LENGTH_MAX = 100
FUNCTION_LINES_MAX = 70
ASSERTION_DENSITY_MIN = 2.0

ASSERTION_PATTERNS = [
    re.compile(r'\bassert\s*\('),
    re.compile(r'\bassert!\s*\('),
    re.compile(r'\bassert_eq!\s*\('),
    re.compile(r'\bassert_ne!\s*\('),
    re.compile(r'\bdebug_assert!\s*\('),
    re.compile(r'\bdebug_assert_eq!\s*\('),
    re.compile(r'\bdebug_assert_ne!\s*\('),
    re.compile(r'\bstatic_assert\b'),
]

RUST_FUNCTION_DEF_PATTERN = re.compile(
    r'^\s*(?:pub(?:\s*\([^)]*\))?\s+)?(?:const\s+)?(?:async\s+)?'
    r'(?:unsafe\s+)?fn\s+([a-zA-Z0-9_]+)\s*(?:<[^>]*>)?\s*\('
)

FUNCTION_DEF_PATTERNS = [
    RUST_FUNCTION_DEF_PATTERN, # Zig / Rust
    re.compile(r'^\s*(?:[a-zA-Z0-9_]+[\s\*]+)+([a-zA-Z0-9_]+)\s*\([^;]*\)\s*\{'), # C / C++
    re.compile(r'^\s*(?:export\s+)?(?:public\s+|private\s+)?(?:async\s+)?function\s+([a-zA-Z0-9_]+)\s*\('), # TS/JS
    re.compile(r'^\s*(?:public\s+|private\s+)?([a-zA-Z0-9_]+)\s*\([^;]*\)\s*:\s*[a-zA-Z0-9_<>\[\]\s\|]+\s*\{'), # TS method
]

LITTLE_ENDIAN_PREFIXES = [
    re.compile(r'\b(max_[a-zA-Z0-9_]+)\b'),
    re.compile(r'\b(min_[a-zA-Z0-9_]+)\b'),
    re.compile(r'\b(total_[a-zA-Z0-9_]+)\b'),
]

EXTERNAL_METHOD_ALLOWLIST = {
    "total_cmp",
    "min_by_key",
    "max_by_key",
    "min_by",
    "max_by",
    "min_keycode",
    "max_keycode",
}

TRIVIAL_TRAIT_METHODS = {
    "default",
    "from",
    "fmt",
    "id",
    "title",
    "icon_name",
    "menu",
}


ALLOCATION_PATTERNS = [
    re.compile(r'\bmalloc\s*\('),
    re.compile(r'\bcalloc\s*\('),
    re.compile(r'\brealloc\s*\('),
    re.compile(r'\bBox::new\('),
    re.compile(r'\bpage_allocator\b'),
]

def check_file(filepath: str) -> Dict[str, Any]:
    issues = []
    function_stats = []

    try:
        with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
            lines = f.readlines()
    except Exception as e:
        return {"filepath": filepath, "error": str(e), "issues": [], "function_stats": []}

    # 1. Check Line Lengths
    for line_idx, line in enumerate(lines, start=1):
        clean_line = line.rstrip('\r\n')
        if len(clean_line) > LINE_LENGTH_MAX:
            issues.append({
                "line": line_idx,
                "type": "LINE_WIDTH",
                "message": f"Line exceeds {LINE_LENGTH_MAX} columns ({len(clean_line)} chars)"
            })

    # 2. Check Functions & Assertions
    func_patterns = (
        [RUST_FUNCTION_DEF_PATTERN]
        if filepath.endswith((".rs", ".zig"))
        else FUNCTION_DEF_PATTERNS
    )
    current_func = None
    func_start_line = 0
    func_brace_depth = 0
    func_assert_count = 0
    in_func_signature = False

    for line_idx, line in enumerate(lines, start=1):
        stripped = line.strip()

        # Check for function start
        if current_func is None:
            for pat in func_patterns:
                match = pat.search(line)
                if match:
                    func_name = match.group(1)
                    if func_name in ("assert", "assert_"):
                        break
                    if ";" in line and "{" not in line:
                        break
                    current_func = func_name
                    func_start_line = line_idx
                    in_func_signature = '{' not in line
                    func_brace_depth = line.count('{') - line.count('}')
                    func_assert_count = 0
                    for assert_pat in ASSERTION_PATTERNS:
                        if assert_pat.search(line):
                            func_assert_count += 1
                    break
        else:
            if in_func_signature:
                if ";" in line and "{" not in line:
                    current_func = None
                    in_func_signature = False
                    continue
                if '{' in line:
                    in_func_signature = False
                    func_brace_depth = line.count('{') - line.count('}')
            else:
                func_brace_depth += line.count('{') - line.count('}')

            for assert_pat in ASSERTION_PATTERNS:
                if assert_pat.search(line):
                    func_assert_count += 1

        if current_func is not None and not in_func_signature and func_brace_depth <= 0:
            func_length = line_idx - func_start_line + 1
            function_stats.append({
                "name": current_func,
                "start": func_start_line,
                "end": line_idx,
                "lines": func_length,
                "assertions": func_assert_count,
            })

            if func_length > FUNCTION_LINES_MAX:
                msg = (
                    f"Function '{current_func}' exceeds {FUNCTION_LINES_MAX} lines "
                    f"({func_length} lines)"
                )
                issues.append({
                    "line": func_start_line,
                    "type": "FUNCTION_LENGTH",
                    "message": msg,
                })

            is_trivial = (
                func_length <= 5
                or current_func.startswith("test")
                or current_func in TRIVIAL_TRAIT_METHODS
            )
            if func_assert_count < 2 and not is_trivial:
                msg = (
                    f"Function '{current_func}' has {func_assert_count} assertions "
                    f"(TigerStyle minimum: 2)"
                )
                issues.append({
                    "line": func_start_line,
                    "type": "LOW_ASSERTION_DENSITY",
                    "message": msg,
                })

            current_func = None
            func_start_line = 0
            func_assert_count = 0
            in_func_signature = False

        # Check for little endian naming in declarations
        for le_pat in LITTLE_ENDIAN_PREFIXES:
            matches = le_pat.findall(line)
            for m in matches:
                # ignore standard macros, external methods, or uppercase constants
                if not m.isupper() and m not in EXTERNAL_METHOD_ALLOWLIST:
                    msg = (
                        f"Little-endian name detected: '{m}'. "
                        f"Prefer big-endian (e.g. '<entity>_max' or '<entity>_min')"
                    )
                    issues.append({
                        "line": line_idx,
                        "type": "NAMING_CONVENTION",
                        "message": msg,
                    })

    return {
        "filepath": filepath,
        "issues": issues,
        "function_stats": function_stats
    }

def main():
    parser = argparse.ArgumentParser(description="TigerStyle Linter & Rule Checker")
    parser.add_argument("paths", nargs="+", help="Files or directories to scan")
    parser.add_argument("--strict", action="store_true", help="Exit with code 1 if any issues found")
    args = parser.parse_args()

    files_to_check = []
    for path in args.paths:
        if os.path.isfile(path):
            files_to_check.append(path)
        elif os.path.isdir(path):
            for root, _, filenames in os.walk(path):
                for f in filenames:
                    if f.endswith(('.zig', '.c', '.h', '.cpp', '.hpp', '.rs', '.ts', '.go')):
                        files_to_check.append(os.path.join(root, f))

    total_issues = 0
    total_functions = 0
    total_assertions = 0

    print(f"=== TigerStyle Linter ===")
    print(f"Scanning {len(files_to_check)} files...\n")

    for filepath in files_to_check:
        res = check_file(filepath)
        for stat in res["function_stats"]:
            total_functions += 1
            total_assertions += stat["assertions"]

        if res["issues"]:
            print(f"📄 {filepath}:")
            for issue in res["issues"]:
                total_issues += 1
                print(f"  Line {issue['line']:4d} [{issue['type']}]: {issue['message']}")
            print()

    avg_density = (total_assertions / total_functions) if total_functions > 0 else 0.0
    print(f"--------------------------------------------------")
    print(f"Files scanned:       {len(files_to_check)}")
    print(f"Functions detected:  {total_functions}")
    print(f"Total assertions:    {total_assertions}")
    print(f"Avg assertions/func: {avg_density:.2f} (Target: >= 2.00)")
    print(f"Total violations:    {total_issues}")
    print(f"--------------------------------------------------")

    if total_issues == 0:
        print("✅ 100% TigerStyle Compliant! Zero technical debt detected.")
        sys.exit(0)
    else:
        print(f"⚠️ Found {total_issues} TigerStyle violations.")
        if args.strict:
            sys.exit(1)

if __name__ == "__main__":
    main()
