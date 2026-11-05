//! Exact `_MAX` boundary transition tests (`0`, `MAX - 1`, `MAX`, `MAX + 1`) for `pak_vfs`.

use pak::{ReadScratch, ScratchLimits};

use crate::tests::builder::{TempDir, TestPakEntry, TestPakSpec};
use crate::{
    ARENA_BYTES_HARD_MAX, DIRS_COUNT_HARD_MAX, DirId, EXTS_COUNT_HARD_MAX, FILES_COUNT_HARD_MAX,
    MOUNT_ALIASES_COUNT_HARD_MAX, MountSource, PAKS_COUNT_HARD_MAX, PATH_DEPTH_HARD_MAX, PakVfs,
    VFS_PATH_BYTES_HARD_MAX, VfsError, VfsLimitKind, VfsLimits, VfsOpenOptions,
};

#[test]
fn vfs_limits_zero_and_hard_max_boundaries() {
    let d = VfsLimits::DEFAULT;
    for res in [
        d.with_paks_count_max(0),
        d.with_dirs_count_max(0),
        d.with_files_count_max(0),
        d.with_exts_count_max(0),
        d.with_arena_bytes_max(0),
        d.with_path_bytes_max(0),
        d.with_path_depth_max(0),
        d.with_mount_aliases_count_max(0),
    ] {
        assert!(matches!(
            res,
            Err(VfsError::LimitExceeded {
                kind: VfsLimitKind::LimitsConfig,
                ..
            })
        ));
    }

    d.with_paks_count_max(PAKS_COUNT_HARD_MAX).expect("ok");
    d.with_dirs_count_max(DIRS_COUNT_HARD_MAX).expect("ok");
    d.with_files_count_max(FILES_COUNT_HARD_MAX).expect("ok");
    d.with_exts_count_max(EXTS_COUNT_HARD_MAX).expect("ok");
    d.with_arena_bytes_max(ARENA_BYTES_HARD_MAX).expect("ok");
    d.with_path_bytes_max(VFS_PATH_BYTES_HARD_MAX).expect("ok");
    d.with_path_depth_max(PATH_DEPTH_HARD_MAX).expect("ok");
    d.with_mount_aliases_count_max(MOUNT_ALIASES_COUNT_HARD_MAX)
        .expect("ok");

    for (res, expected_kind) in [
        (
            d.with_dirs_count_max(DIRS_COUNT_HARD_MAX + 1),
            VfsLimitKind::DirsCount,
        ),
        (
            d.with_files_count_max(FILES_COUNT_HARD_MAX + 1),
            VfsLimitKind::FilesCount,
        ),
        (
            d.with_exts_count_max(EXTS_COUNT_HARD_MAX + 1),
            VfsLimitKind::ExtsCount,
        ),
        (
            d.with_arena_bytes_max(ARENA_BYTES_HARD_MAX + 1),
            VfsLimitKind::ArenaBytes,
        ),
        (
            d.with_path_bytes_max(VFS_PATH_BYTES_HARD_MAX + 1),
            VfsLimitKind::PathBytes,
        ),
        (
            d.with_path_depth_max(PATH_DEPTH_HARD_MAX + 1),
            VfsLimitKind::PathDepth,
        ),
        (
            d.with_mount_aliases_count_max(MOUNT_ALIASES_COUNT_HARD_MAX + 1),
            VfsLimitKind::MountAliasesCount,
        ),
    ] {
        assert!(matches!(
            res,
            Err(VfsError::LimitExceeded { kind, .. }) if kind == expected_kind
        ));
    }
}

#[test]
fn empty_open_mounts_is_total() {
    let vfs = PakVfs::open_mounts(&[], &VfsOpenOptions::new(None)).expect("empty mounts valid");
    assert_eq!(vfs.pak_count(), 0);
    assert_eq!(vfs.file_count(), 0);
    assert_eq!(vfs.dir_count(), 1);
    assert_eq!(vfs.dir_id_at(0), Some(DirId::ROOT));
    assert_eq!(vfs.dir_id_at(1), None);
    assert_eq!(vfs.pak_id_at(0), None);
    assert_eq!(vfs.file_id_at(0), None);
    assert_eq!(vfs.find("any/file.txt").expect("ok"), None);
}

#[test]
fn paks_files_dirs_and_exts_exact_max_transitions() {
    let tmp = TempDir::new("count_bounds");
    let mut s1 = TestPakSpec::new_v12("../../../");
    s1.entries.push(TestPakEntry::plain("A", "f1.e1", b"1"));
    s1.entries.push(TestPakEntry::plain("B", "f2.e2", b"2"));
    let p1 = tmp.write_pak("1.pak", &s1);

    let mut s2 = TestPakSpec::new_v12("../../../");
    s2.entries.push(TestPakEntry::plain("C", "f3.e3", b"3"));
    let p2 = tmp.write_pak("2.pak", &s2);
    let mounts = [MountSource::new(&p1), MountSource::new(&p2)];

    // PaksCount: 2 succeeds at max=2, fails at max=1.
    let lim_p2 = VfsLimits::DEFAULT.with_paks_count_max(2).expect("ok");
    PakVfs::open_mounts(&mounts, &VfsOpenOptions::new(None).with_limits(lim_p2)).expect("p2 ok");
    let lim_p1 = VfsLimits::DEFAULT.with_paks_count_max(1).expect("ok");
    assert!(matches!(
        PakVfs::open_mounts(&mounts, &VfsOpenOptions::new(None).with_limits(lim_p1)),
        Err(VfsError::LimitExceeded {
            kind: VfsLimitKind::PaksCount,
            ..
        })
    ));

    // FilesCount: 3 files total succeeds at max=3, fails at max=2.
    let lim_f3 = VfsLimits::DEFAULT.with_files_count_max(3).expect("ok");
    PakVfs::open_mounts(&mounts, &VfsOpenOptions::new(None).with_limits(lim_f3)).expect("f3 ok");
    let lim_f2 = VfsLimits::DEFAULT.with_files_count_max(2).expect("ok");
    assert!(matches!(
        PakVfs::open_mounts(&mounts, &VfsOpenOptions::new(None).with_limits(lim_f2)),
        Err(VfsError::LimitExceeded {
            kind: VfsLimitKind::FilesCount,
            ..
        })
    ));

    // DirsCount: ROOT + A + B + C = 4 dirs succeeds at max=4, fails at max=3.
    let lim_d4 = VfsLimits::DEFAULT.with_dirs_count_max(4).expect("ok");
    PakVfs::open_mounts(&mounts, &VfsOpenOptions::new(None).with_limits(lim_d4)).expect("d4 ok");
    let lim_d3 = VfsLimits::DEFAULT.with_dirs_count_max(3).expect("ok");
    assert!(matches!(
        PakVfs::open_mounts(&mounts, &VfsOpenOptions::new(None).with_limits(lim_d3)),
        Err(VfsError::LimitExceeded {
            kind: VfsLimitKind::DirsCount,
            ..
        })
    ));

    // ExtsCount: e1, e2, e3 = 3 exts succeeds at max=3, fails at max=2.
    let lim_e3 = VfsLimits::DEFAULT.with_exts_count_max(3).expect("ok");
    PakVfs::open_mounts(&mounts, &VfsOpenOptions::new(None).with_limits(lim_e3)).expect("e3 ok");
    let lim_e2 = VfsLimits::DEFAULT.with_exts_count_max(2).expect("ok");
    assert!(matches!(
        PakVfs::open_mounts(&mounts, &VfsOpenOptions::new(None).with_limits(lim_e2)),
        Err(VfsError::LimitExceeded {
            kind: VfsLimitKind::ExtsCount,
            ..
        })
    ));
}

#[test]
fn path_depth_path_bytes_arena_and_alias_max_transitions() {
    let tmp = TempDir::new("path_bounds");
    // Path "A/B/C/file.tx" has depth=3 ("A", "B", "C") and normalized byte length 13.
    let mut spec = TestPakSpec::new_v12("../../../");
    spec.entries
        .push(TestPakEntry::plain("A/B/C", "file.tx", b"ok"));
    spec.entries
        .push(TestPakEntry::plain("Engine/Content", "e.tx", b"ok"));
    spec.entries
        .push(TestPakEntry::plain("GameProj/Content", "g.tx", b"ok"));
    let p = tmp.write_pak("b.pak", &spec);
    let m = [MountSource::new(&p)];

    // PathDepth: max depth across entries is 3 -> succeeds at 3, fails at 2.
    let lim_dep3 = VfsLimits::DEFAULT.with_path_depth_max(3).expect("ok");
    let vfs = PakVfs::open_mounts(&m, &VfsOpenOptions::new(None).with_limits(lim_dep3))
        .expect("depth 3 ok");
    let exact_arena = vfs.arena_bytes();
    let lim_dep2 = VfsLimits::DEFAULT.with_path_depth_max(2).expect("ok");
    assert!(matches!(
        PakVfs::open_mounts(&m, &VfsOpenOptions::new(None).with_limits(lim_dep2)),
        Err(VfsError::LimitExceeded {
            kind: VfsLimitKind::PathDepth,
            ..
        })
    ));

    // ArenaBytes: succeeds at `exact_arena`, fails at `exact_arena - 1`.
    let lim_ar_ok = VfsLimits::DEFAULT
        .with_arena_bytes_max(exact_arena)
        .expect("ok");
    PakVfs::open_mounts(&m, &VfsOpenOptions::new(None).with_limits(lim_ar_ok)).expect("ar ok");
    let lim_ar_fail = VfsLimits::DEFAULT
        .with_arena_bytes_max(exact_arena - 1)
        .expect("ok");
    assert!(matches!(
        PakVfs::open_mounts(&m, &VfsOpenOptions::new(None).with_limits(lim_ar_fail)),
        Err(VfsError::LimitExceeded {
            kind: VfsLimitKind::ArenaBytes,
            ..
        })
    ));

    // PathBytes: longest entry "GameProj/Content/g.tx" is 21 bytes -> succeeds at 21, fails at 20.
    let lim_pb21 = VfsLimits::DEFAULT.with_path_bytes_max(21).expect("ok");
    PakVfs::open_mounts(&m, &VfsOpenOptions::new(None).with_limits(lim_pb21)).expect("pb21 ok");
    let lim_pb20 = VfsLimits::DEFAULT.with_path_bytes_max(20).expect("ok");
    assert!(matches!(
        PakVfs::open_mounts(&m, &VfsOpenOptions::new(None).with_limits(lim_pb20)),
        Err(VfsError::LimitExceeded {
            kind: VfsLimitKind::PathBytes,
            ..
        })
    ));

    // MountAliasesCount: `/Engine` and `/Game` = 2 aliases -> succeeds at 2, fails at 1.
    let lim_ma2 = VfsLimits::DEFAULT
        .with_mount_aliases_count_max(2)
        .expect("ok");
    PakVfs::open_mounts(&m, &VfsOpenOptions::new(None).with_limits(lim_ma2)).expect("ma2 ok");
    let lim_ma1 = VfsLimits::DEFAULT
        .with_mount_aliases_count_max(1)
        .expect("ok");
    assert!(matches!(
        PakVfs::open_mounts(&m, &VfsOpenOptions::new(None).with_limits(lim_ma1)),
        Err(VfsError::LimitExceeded {
            kind: VfsLimitKind::MountAliasesCount,
            ..
        })
    ));
}

#[test]
fn read_range_exact_eof_boundaries() {
    let tmp = TempDir::new("range_bounds");
    let mut spec = TestPakSpec::new_v12("../../../");
    spec.entries
        .push(TestPakEntry::plain("Dir", "Four.bin", b"ABCD"));
    let p = tmp.write_pak("r.pak", &spec);
    let vfs =
        PakVfs::open_mounts(&[MountSource::new(&p)], &VfsOpenOptions::new(None)).expect("open");
    let fid = vfs.find("Dir/Four.bin").expect("ok").expect("found");
    let mut scratch = ReadScratch::with_limits(ScratchLimits::DEFAULT);

    // 0-byte read at exact EOF (offset == 4) must succeed; offset == 5 or 1 byte at offset == 4 must fail.
    vfs.read_range_into(fid, 4, &mut scratch, &mut [])
        .expect("0-byte read at EOF ok");
    assert!(matches!(
        vfs.read_range_into(fid, 5, &mut scratch, &mut []),
        Err(VfsError::OutOfBoundsRange { .. })
    ));
    let mut one = [0_u8; 1];
    assert!(matches!(
        vfs.read_range_into(fid, 4, &mut scratch, &mut one),
        Err(VfsError::OutOfBoundsRange { .. })
    ));
    assert!(matches!(
        vfs.read_range_into(fid, u64::MAX, &mut scratch, &mut one),
        Err(VfsError::OutOfBoundsRange { .. })
    ));
}

#[test]
fn ue_mount_alias_and_project_name_path_byte_bounds() {
    let lim_pb10 = VfsLimits::DEFAULT.with_path_bytes_max(10).expect("ok");
    let vfs =
        PakVfs::open_mounts(&[], &VfsOpenOptions::new(None).with_limits(lim_pb10)).expect("open");
    assert!(matches!(
        vfs.find_ue("/AliasOver10Bytes/f.tx"),
        Err(VfsError::LimitExceeded {
            kind: VfsLimitKind::PathBytes,
            ..
        })
    ));

    let max_proj = "p".repeat(crate::u32_to_usize(VFS_PATH_BYTES_HARD_MAX));
    let _opts = VfsOpenOptions::new(None)
        .with_game_project_name(&max_proj)
        .expect("max_proj ok");
    let over_proj = "p".repeat(crate::u32_to_usize(VFS_PATH_BYTES_HARD_MAX + 1));
    assert!(matches!(
        VfsOpenOptions::new(None).with_game_project_name(&over_proj),
        Err(VfsError::InvalidProjectName)
    ));
}
