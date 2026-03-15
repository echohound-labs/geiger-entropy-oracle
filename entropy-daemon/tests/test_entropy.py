"""
Unit tests for Geiger Entropy Oracle daemon entropy logic.
"""
import hashlib
import struct
import sys
from pathlib import Path

# Add parent to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from daemon import (
    compute_quality_score,
    extract_entropy_from_cps,
    get_latest_row,
    parse_gmc_csv,
    xor_seeds,
)


# ---------------------------------------------------------------------------
# Sample CSV (matches real GMC-500 output format)
# ---------------------------------------------------------------------------

SAMPLE_CSV = """\
GQ Electronics LLC, GMC Data Viewer,Version 2.75
Date Time,Type,uSv/h,CPM,#1(CPS),#2(CPS),#3(CPS),#4(CPS),#5(CPS),#6(CPS),#7(CPS),#8(CPS),#9(CPS),#10(CPS),#11(CPS),#12(CPS),#13(CPS),#14(CPS),#15(CPS),#16(CPS),#17(CPS),#18(CPS),#19(CPS),#20(CPS),#21(CPS),#22(CPS),#23(CPS),#24(CPS),#25(CPS),#26(CPS),#27(CPS),#28(CPS),#29(CPS),#30(CPS),#31(CPS),#32(CPS),#33(CPS),#34(CPS),#35(CPS),#36(CPS),#37(CPS),#38(CPS),#39(CPS),#40(CPS),#41(CPS),#42(CPS),#43(CPS),#44(CPS),#45(CPS),#46(CPS),#47(CPS),#48(CPS),#49(CPS),#50(CPS),#51(CPS),#52(CPS),#53(CPS),#54(CPS),#55(CPS),#56(CPS),#57(CPS),#58(CPS),#59(CPS),#60(CPS),

2026-03-15 08:16,Every Second,0.117,18,0,0,0,1,0,0,0,0,0,0,0,1,1,1,1,0,0,0,0,0,1,0,0,0,0,0,1,0,0,0,0,0,0,0,0,1,1,0,2,1,0,2,0,0,0,0,0,0,1,0,0,1,1,0,0,1,0,0,0,0,
2026-03-15 08:17,Every Second,0.104,16,0,0,0,0,0,0,0,1,0,0,0,1,0,0,0,0,0,0,0,1,1,0,0,0,0,0,2,0,2,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1,1,0,1,0,0,0,0,1,1,1,
2026-03-15 08:18,Every Second,0.130,20,0,0,0,0,0,0,0,0,0,0,0,0,1,0,1,0,1,0,0,0,0,0,0,1,1,1,0,1,0,0,0,0,0,1,0,1,0,0,1,0,1,1,2,0,0,0,0,0,0,0,0,1,1,0,1,0,0,3,0,0,
"""


def write_sample_csv(tmp_path: Path) -> Path:
    p = tmp_path / "sample.csv"
    p.write_text(SAMPLE_CSV)
    return p


# ---------------------------------------------------------------------------
# Tests
# ---------------------------------------------------------------------------

def test_extract_entropy_basic():
    cps = [0, 1, 2, 0, 0, 1, 3, 0, 0, 1] * 6  # 60 values
    seed = extract_entropy_from_cps(cps)
    assert len(seed) == 32, "Seed must be 32 bytes (SHA-256)"
    assert isinstance(seed, bytes)


def test_extract_entropy_deterministic():
    cps = list(range(60))
    seed1 = extract_entropy_from_cps(cps)
    seed2 = extract_entropy_from_cps(cps)
    assert seed1 == seed2, "Same input must produce same seed"


def test_extract_entropy_different_inputs():
    cps1 = [1] * 60
    cps2 = [2] * 60
    assert extract_entropy_from_cps(cps1) != extract_entropy_from_cps(cps2)


def test_extract_entropy_matches_sha256():
    cps = [0, 1, 2, 3, 4] * 12  # 60 values
    raw = struct.pack(f"<{len(cps)}H", *cps)
    expected = hashlib.sha256(raw).digest()
    assert extract_entropy_from_cps(cps) == expected


def test_quality_score_range():
    for cpm in [0, 5, 20, 100, 500]:
        cps = [cpm // 60] * 60
        score = compute_quality_score(cpm, cps)
        assert 0.0 <= score <= 1.0, f"Quality score out of range for CPM={cpm}"


def test_quality_score_zero_cpm():
    assert compute_quality_score(0, []) == 0.0


def test_quality_score_higher_cpm_better():
    score_low = compute_quality_score(10, [0] * 60)
    score_high = compute_quality_score(200, [3] * 60)
    assert score_high > score_low


def test_xor_seeds_identity():
    seed = bytes(range(32))
    assert xor_seeds([seed]) == seed


def test_xor_seeds_self_cancels():
    seed = bytes(range(32))
    result = xor_seeds([seed, seed])
    assert result == b"\x00" * 32


def test_xor_seeds_associative():
    a = bytes([i % 256 for i in range(32)])
    b = bytes([(i * 3) % 256 for i in range(32)])
    c = bytes([(i * 7) % 256 for i in range(32)])
    assert xor_seeds([a, b, c]) == xor_seeds([c, a, b])


def test_xor_seeds_empty():
    assert xor_seeds([]) == b"\x00" * 32


def test_parse_csv(tmp_path):
    p = write_sample_csv(tmp_path)
    rows = parse_gmc_csv(p)
    assert len(rows) == 3
    assert rows[0]["datetime"] == "2026-03-15 08:16"
    assert rows[0]["cpm"] == 18
    assert len(rows[0]["cps_values"]) == 60
    assert rows[0]["usv_h"] == 0.117


def test_parse_csv_cps_values(tmp_path):
    p = write_sample_csv(tmp_path)
    rows = parse_gmc_csv(p)
    # First row CPS values should sum to CPM
    assert sum(rows[0]["cps_values"]) == rows[0]["cpm"]


def test_get_latest_row(tmp_path):
    p = write_sample_csv(tmp_path)
    row = get_latest_row(p)
    assert row is not None
    assert row["datetime"] == "2026-03-15 08:18"  # last row


def test_entropy_from_real_sample(tmp_path):
    """Full pipeline test: parse CSV → extract entropy → check properties."""
    p = write_sample_csv(tmp_path)
    row = get_latest_row(p)
    assert row is not None
    seed = extract_entropy_from_cps(row["cps_values"])
    assert len(seed) == 32
    # Seed should not be all zeros
    assert any(b != 0 for b in seed)


if __name__ == "__main__":
    import tempfile
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        test_extract_entropy_basic()
        test_extract_entropy_deterministic()
        test_extract_entropy_different_inputs()
        test_extract_entropy_matches_sha256()
        test_quality_score_range()
        test_quality_score_zero_cpm()
        test_quality_score_higher_cpm_better()
        test_xor_seeds_identity()
        test_xor_seeds_self_cancels()
        test_xor_seeds_associative()
        test_xor_seeds_empty()
        test_parse_csv(tmp_path)
        test_parse_csv_cps_values(tmp_path)
        test_get_latest_row(tmp_path)
        test_entropy_from_real_sample(tmp_path)
        print("✅ All tests passed!")
