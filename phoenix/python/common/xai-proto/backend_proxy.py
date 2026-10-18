# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
from hatchling.build import build_wheel as _bw
from hatchling.build import get_requires_for_build_wheel as _grw
from hatchling.build import prepare_metadata_for_build_wheel as _pmw


def build_wheel(wheel_directory, config_settings=None, metadata_directory=None):
    return _bw(wheel_directory, config_settings, metadata_directory)


def get_requires_for_build_wheel(config_settings=None):
    return _grw(config_settings)


def prepare_metadata_for_build_wheel(metadata_directory, config_settings=None):
    return _pmw(metadata_directory, config_settings)


def build_editable(wheel_directory, config_settings=None, metadata_directory=None):
    return build_wheel(wheel_directory, config_settings, metadata_directory)


def get_requires_for_build_editable(config_settings=None):
    return get_requires_for_build_wheel(config_settings)


def prepare_metadata_for_build_editable(metadata_directory, config_settings=None):
    return prepare_metadata_for_build_wheel(metadata_directory, config_settings)
