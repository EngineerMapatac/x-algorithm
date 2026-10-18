# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 X.AI Corp.
from __future__ import annotations

import numpy as np
import numpy.typing as npt

NUM_TOPIC_BITS = 86
NUM_TOPIC_INT32S = 3

_CATEGORIES: dict[int, tuple[str, list[int]]] = {
    0: ("science_technology", [1000000000000000001]),
    1: ("entertainment", [1000000000000000002]),
    3: ("business_finance", [1000000000000000003]),
    4: ("sports", [1000000000000000004]),
    2: ("politics", [1925952771733262336]),
    5: ("ai", [1925953013547450368]),
    6: ("gaming", [1925949766673797120]),
    7: ("crypto", [1925949693290295298]),
    8: ("us_iran_war", [2000000000000000001]),
    9: ("news", [1925949634972626944, 1925952795854745601]),
    10: ("movies_tv", [1925949788068909057, 1925950455806369792]),
    11: ("music", [1925949604224196608]),
    12: ("dance", [1925950383899267072]),
    13: ("celebrity", [1925949555071176704]),
    14: ("anime", [1925949503837798401]),
    15: (
        "soccer",
        [
            1925950498420469760,
            1925951257107206144,
            1925951079100928001,
            1925951305895272448,
            1925951334777233408,
            1925951366175813632,
            1925951417010753536,
        ],
    ),
    16: ("basketball", [1925950530842464256, 1925951454121967616, 1925951561211023361]),
    17: ("american_football", [1925950554133446656, 1925951645872975873, 1925951614067650560]),
    18: ("baseball", [1925950576740675584, 1925951672561319936]),
    19: ("tennis", [1925950625835008001]),
    20: ("cricket", [1925950653035151360, 1925951866464006145]),
    21: ("mma", [1925950600140718080, 1925950864117702656]),
    22: ("boxing", [1925950713982537888]),
    23: ("golf", [1925950690574180352]),
    24: ("racing", [1943810034309246976, 1925950746756800513, 1925951703007858688]),
    25: ("ice_hockey", [1925950813043662848, 1925951775544061952]),
    26: ("olympics", [1943811470724153345]),
    27: ("rugby", [1925950895595876352]),
    28: ("cycling", [1925950926910582786]),
    29: ("skiing", [1925950954962096130, 1925950982346682368]),
    30: ("health_fitness", [1925949856834588672, 1925953366091350017, 1925953337020555264]),
    31: ("travel", [1925949812844769280, 1925950031476957184]),
    32: (
        "food",
        [
            1925949835649126400,
            1925953199694819328,
            1925953282146451456,
            1925953243223339009,
            1925953310894252033,
        ],
    ),
    33: ("memes", [1925949876694556672]),
    34: ("beauty", [1925950199861538818]),
    35: ("fashion", [1925949932181082112]),
    36: ("nature_outdoors", [1925950228047216640]),
    37: ("pets", [1925950249245282304, 1925953794732433408, 1925953826281902080]),
    38: ("home_garden", [1925950341452845057]),
    39: ("art", [1925949898106540032]),
    40: ("religion", [1925949956008878080]),
    41: ("shopping", [1925949979966701568]),
    42: ("education", [1925950405709631490]),
    43: ("career", [1925950361245786112]),
    44: ("cars", [1925950007624028160]),
    45: ("motorcycles", [1925950100397793280]),
    46: ("relationships", [1925950276835454977]),
    47: ("family", [1925954059338493952, 1925953930002907137, 1925954095342338048]),
    48: ("podcasts", [1925950433807175680]),
    49: ("pop", [1925952179791151105]),
    50: ("k_pop", [1925952056088530944]),
    51: ("country_music", [1925952353485602816]),
    52: ("electronic", [1925952257893257217]),
    53: ("j_pop", [1925952123960709120]),
    54: ("rock", [1925952228818354177]),
    55: ("photography", [1925954143472017408]),
    56: ("design", [1925954681496322048]),
    57: ("software_development", [1925953040130953216]),
    58: ("robotics", [1925953169307189248]),
    59: ("space", [1925953141452718081]),
    60: ("biotech", [1925953107038519296]),
    61: ("electronics", [1925953073983139840]),
    62: ("stocks", [1925952876284727296]),
    63: ("entrepreneurship", [1925952916411564032]),
    64: ("real_estate", [1925952947197784064]),
    65: ("personal_finance", [1925952988486447104]),
    66: ("crime", [1925952744239620096]),
    67: ("dating", [1925953884108824576]),
    68: ("elections", [1925952820714332161]),
    69: ("esports", [1925950783448576000]),
    70: ("hip_hop", [1925952203962896384]),
    71: ("jazz", [1925952300213768192]),
    72: ("concerts", [1925952145615974400]),
    73: ("mlb", [1925951672561319936]),
    74: ("mental_health", [1925953421338726400]),
    75: ("moto_gp", [1925951732204335105]),
    76: ("premier_league", [1925951020036751361]),
    77: ("serie_a", [1925951109660577792]),
    78: ("christianity", [1925953452644978688]),
    79: ("buddhism", [1925953574929842176]),
    80: ("hinduism", [1925953483758338048]),
    81: ("islam", [1925953516834590721]),
    82: ("judaism", [1925953757117865984]),
    83: ("digital_art", [1925954197326831616]),
    84: ("stocks_and_economy", [2000000000000000002]),
    85: ("xai_business_finance", [1925949659857530880]),
}

_POST_SIDE_GROUP_MEMBERS: dict[int, frozenset[int]] = {
    1925949744683114496: frozenset({0}),
    1925949722688126976: frozenset({0}),
    1925953013547450368: frozenset({0}),
    1925949788068909057: frozenset({1}),
    1925949604224196608: frozenset({1}),
    1925949555071176704: frozenset({1}),
    1925949766673797120: frozenset({1}),
    1925949503837798401: frozenset({1}),
    1925950455806369792: frozenset({1}),
    1925950383899267072: frozenset({1}),
    1925949659857530880: frozenset({3}),
    1925949693290295298: frozenset({3}),
    1925949452197478400: frozenset({4}),
    1925950838414942210: frozenset({4}),
}

TOPIC_ID_TO_BITS: dict[int, frozenset[int]] = {}
for _bit, (_, _eids) in _CATEGORIES.items():
    for _eid in _eids:
        TOPIC_ID_TO_BITS[_eid] = TOPIC_ID_TO_BITS.get(_eid, frozenset()) | frozenset({_bit})

_all_bits = set(_CATEGORIES.keys())
assert max(_all_bits) < NUM_TOPIC_BITS, (
    f"bit index {max(_all_bits)} exceeds NUM_TOPIC_BITS={NUM_TOPIC_BITS}; "
    f"bump NUM_TOPIC_BITS and NUM_TOPIC_INT32S accordingly."
)
_names = [name for _, (name, _) in _CATEGORIES.items()]
assert len(_names) == len(set(_names)), f"duplicate category name in _CATEGORIES: {_names}"
for _tid, _bits in _POST_SIDE_GROUP_MEMBERS.items():
    _unknown = _bits - _all_bits
    assert not _unknown, f"_POST_SIDE_GROUP_MEMBERS[{_tid}] references unknown bits {_unknown}"
del _all_bits, _names


def topic_ids_to_bitmap(topic_entity_ids: list[int] | None) -> int:
    if not topic_entity_ids:
        return 0
    bitmap = 0
    for topic_id in topic_entity_ids:
        for bit in TOPIC_ID_TO_BITS.get(topic_id, ()):
            bitmap |= 1 << bit
        for bit in _POST_SIDE_GROUP_MEMBERS.get(topic_id, ()):
            bitmap |= 1 << bit
    return bitmap


def bitmaps_to_int32_array(bitmaps: list[int]) -> npt.NDArray[np.int32]:
    mask32 = (1 << 32) - 1
    w0 = np.array([b & mask32 for b in bitmaps], dtype=np.uint32).view(np.int32)
    w1 = np.array([(b >> 32) & mask32 for b in bitmaps], dtype=np.uint32).view(np.int32)
    w2 = np.array([(b >> 64) & mask32 for b in bitmaps], dtype=np.uint32).view(np.int32)
    return np.stack([w0, w1, w2], axis=-1)
