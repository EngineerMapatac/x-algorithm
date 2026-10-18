# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
def plural(sequence_or_int):
    if not isinstance(sequence_or_int, int):
        sequence_or_int = len(sequence_or_int)
    if sequence_or_int == 1:
        return ""
    return "s"


def contract(numbers, minimum=4):
    numbers = sorted(numbers)
    if len(numbers) < minimum:
        return str(numbers)
    ranges = []
    start = prev = numbers[0]
    for num in numbers[1:] + [prev + 2]:
        if num != prev + 1:
            if start == prev:
                ranges.append(str(start))
            elif prev == start + 1:
                ranges.append(f"{start}, {prev}")
            else:
                ranges.append(f"{start}-{prev}")
            start = num
        prev = num
    return f"[{len(numbers)}]{{{', '.join(ranges)}}}"


def color(s, value=None):
    if value is None:
        return s
    return "\033[%d;3%dm%s\033[0m" % (bool(value & 8), value & ~8, s)
