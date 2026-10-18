# BSD 3-Clause License
#
# Copyright (c) 2022, the respective contributors, as shown by the AUTHORS file.
# All rights reserved.
#
# Redistribution and use in source and binary forms, with or without
# modification, are permitted provided that the following conditions are met:
#
# * Redistributions of source code must retain the above copyright notice, this
#   list of conditions and the following disclaimer.
# * Redistributions in binary form must reproduce the above copyright notice,
#   this list of conditions and the following disclaimer in the documentation
#   and/or other materials provided with the distribution.
# * Neither the name of the copyright holder nor the names of its contributors
#   may be used to endorse or promote products derived from this software without
#   specific prior written permission.
#
# THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
# AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
# IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
# DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
# FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
# DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
# SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
# CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
# OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
# OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
#
# Derived from flash-attention (https://github.com/Dao-AILab/flash-attention);
# modified by X.AI Corp.

# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
import logging
import os
import sys

import cutlass.cute as cute
from cutlass import const_expr

_LOG_LEVEL_NAMES = {"off": 0, "host": 1, "kernel": 2, "max": 3}


def _parse_log_level(raw: str) -> int:
    if raw in _LOG_LEVEL_NAMES:
        return _LOG_LEVEL_NAMES[raw]
    try:
        level = int(raw)
    except ValueError:
        return 0
    return max(0, min(level, 3))


_fa_log_level: int = _parse_log_level(os.environ.get("FA_LOG_LEVEL", "0"))

_logger = logging.getLogger("flash_attn")
_logger.addHandler(logging.NullHandler())
_default_handler: logging.Handler | None = None


def _configure_default_handler() -> None:
    global _default_handler
    if _fa_log_level >= 1:
        if _default_handler is None:
            _default_handler = logging.StreamHandler(sys.stdout)
            _default_handler.setFormatter(logging.Formatter("[FA] %(message)s"))
            _logger.addHandler(_default_handler)
        _logger.setLevel(logging.DEBUG)
    else:
        if _default_handler is not None:
            _logger.removeHandler(_default_handler)
            _default_handler = None
        _logger.setLevel(logging.WARNING)


_configure_default_handler()


def get_fa_log_level() -> int:
    return _fa_log_level


def set_fa_log_level(level: int | str) -> None:
    global _fa_log_level
    if isinstance(level, str):
        level = _parse_log_level(level)
    _fa_log_level = max(0, min(int(level), 3))
    _configure_default_handler()


def fa_log(level: int, msg: str):
    if _fa_log_level >= level:
        _logger.info(msg)


def fa_printf(level: int, fmt, *args):
    if const_expr(_fa_log_level >= level):
        cute.printf(fmt, *args)
