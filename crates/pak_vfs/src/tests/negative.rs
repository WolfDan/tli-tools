//! Negative tests covering malformed paths, namespace conflicts, pruned entries, and contract panics.

use pak::{ReadScratch, ScratchLimits};

use crate::tests::builder::{TempDir, TestPakEntry, TestPakSpec};
use crate::{
    DirId, FileId, MountSource, PakId, PakVfs, PathErrorKind, UePathErrorKind, VfsBatchOutcome,
    VfsError, VfsOpenOptions, VfsReadRequest,
};

#[test]
fn pruned_entry_fails_fast() {
    let tmp = TempDir::new("pruned_reject");
    let mut spec = TestPakSpec::new_v12("../../../");
    let mut pruned = TestPakEntry::plain("Content", "Hidden.uasset", b"secret");
    pruned.prune_directory_path = true;
    spec.entries.push(pruned);
    let p = tmp.write_pak("pruned.pak", &spec);

    let res = PakVfs::open_mounts(&[MountSource::new(&p)], &VfsOpenOptions::new(None));
    assert!(matches!(res, Err(VfsError::PrunedEntryUnsupported { .. })));
}

#[test]
fn path_namespace_conflict_both_directions() {
    let tmp = TempDir::new("ns_conflict");
    let mut file_pak = TestPakSpec::new_v12("../../../");
    file_pak
        .entries
        .push(TestPakEntry::plain("UE_game", "Node", b"file_payload"));
    let fp = tmp.write_pak("file.pak", &file_pak);

    let mut dir_pak = TestPakSpec::new_v12("../../../");
    dir_pak
        .entries
        .push(TestPakEntry::plain("UE_game/Node", "Child.txt", b"child"));
    let dp = tmp.write_pak("dir.pak", &dir_pak);

    // File first, then directory at same path.
    assert!(matches!(
        PakVfs::open_mounts(
            &[MountSource::new(&fp), MountSource::new(&dp)],
            &VfsOpenOptions::new(None),
        ),
        Err(VfsError::PathNamespaceConflict)
    ));
    // Directory first, then file at same path.
    assert!(matches!(
        PakVfs::open_mounts(
            &[MountSource::new(&dp), MountSource::new(&fp)],
            &VfsOpenOptions::new(None),
        ),
        Err(VfsError::PathNamespaceConflict)
    ));
}

#[test]
fn invalid_virtual_and_ue_paths_rejected() {
    let vfs = PakVfs::open_mounts(&[], &VfsOpenOptions::new(None)).expect("empty");

    assert!(matches!(
        vfs.find(""),
        Err(VfsError::InvalidVirtualPath {
            reason: PathErrorKind::EmptyPath
        })
    ));
    assert!(matches!(
        vfs.find("UE_game/Content/"),
        Err(VfsError::InvalidVirtualPath {
            reason: PathErrorKind::TrailingSeparatorOnFile
        })
    ));
    assert!(matches!(
        vfs.find("UE_game/../Secret.ini"),
        Err(VfsError::InvalidVirtualPath {
            reason: PathErrorKind::ParentTraversal
        })
    ));
    assert!(matches!(
        vfs.find("UE_game/./Config.ini"),
        Err(VfsError::InvalidVirtualPath {
            reason: PathErrorKind::DotSegment
        })
    ));
    assert!(matches!(
        vfs.find("UE_game/\0bad.ini"),
        Err(VfsError::InvalidVirtualPath {
            reason: PathErrorKind::ContainsNul
        })
    ));
    assert!(matches!(
        vfs.find_ue("Game/Art/Hero.uasset"),
        Err(VfsError::InvalidUeMountPath {
            reason: UePathErrorKind::MissingLeadingSlash
        })
    ));
    assert!(matches!(
        vfs.find_ue("//Art/Hero.uasset"),
        Err(VfsError::InvalidUeMountPath {
            reason: UePathErrorKind::EmptyMountPoint
        })
    ));
    // Traversal inside an unmounted UE logical alias must still be rejected rather than returning Ok(None).
    assert!(matches!(
        vfs.find_ue("/UnknownPlugin/../../Secret.ini"),
        Err(VfsError::InvalidVirtualPath {
            reason: PathErrorKind::ParentTraversal
        })
    ));
    assert!(matches!(
        VfsOpenOptions::new(None).with_game_project_name("UE/game"),
        Err(VfsError::InvalidProjectName)
    ));
}

#[test]
fn discovery_missing_dirs_empty_paks_and_corrupt_pak_errors() {
    let missing = TempDir::new("missing_content");
    assert!(matches!(
        PakVfs::open_tli_game_dir(missing.path(), None),
        Err(VfsError::MissingContentPaksDir { .. })
    ));

    let empty = TempDir::new("no_paks");
    std::fs::create_dir_all(empty.path().join("UE_game/Content/Paks")).expect("mkdir");
    assert!(matches!(
        PakVfs::open_tli_game_dir(empty.path(), None),
        Err(VfsError::NoPakArchivesFound { .. })
    ));

    let corrupt = TempDir::new("corrupt_pak");
    corrupt.write_raw_file(
        "UE_game/Content/Paks/pakchunk0-WindowsNoEditor.pak",
        b"CORRUPT_TRUNCATED_BYTES",
    );
    assert!(matches!(
        PakVfs::open_tli_game_dir(corrupt.path(), None),
        Err(VfsError::Pak { .. })
    ));
}

#[test]
#[should_panic(expected = "PakId out of bounds")]
fn out_of_bounds_pak_id_panics() {
    let vfs = PakVfs::open_mounts(&[], &VfsOpenOptions::new(None)).expect("empty");
    let _info = vfs.pak_info(PakId::new(0));
}

#[test]
#[should_panic(expected = "DirId out of bounds")]
fn out_of_bounds_dir_id_panics() {
    let vfs = PakVfs::open_mounts(&[], &VfsOpenOptions::new(None)).expect("empty");
    let _view = vfs.read_dir(DirId::new(1));
}

#[test]
#[should_panic(expected = "FileId out of bounds")]
fn out_of_bounds_file_id_panics() {
    let vfs = PakVfs::open_mounts(&[], &VfsOpenOptions::new(None)).expect("empty");
    let _meta = vfs.file_meta(FileId::new(0));
}

#[test]
#[should_panic(expected = "requests and outcomes slices must have identical length")]
fn batch_read_mismatched_outcomes_len_panics() {
    let tmp = TempDir::new("batch_mismatch");
    let mut spec = TestPakSpec::new_v12("../../../");
    spec.entries.push(TestPakEntry::plain("D", "f.bin", b"x"));
    let p = tmp.write_pak("m.pak", &spec);
    let vfs =
        PakVfs::open_mounts(&[MountSource::new(&p)], &VfsOpenOptions::new(None)).expect("open");
    let fid = vfs.file_id_at(0).expect("fid 0");
    let mut dst = [0_u8; 1];
    let mut reqs = [VfsReadRequest::whole_file(fid, &mut dst)];
    let mut outcomes: [VfsBatchOutcome; 0] = [];
    let mut scratch = ReadScratch::with_limits(ScratchLimits::DEFAULT);
    vfs.read_batch_into(&mut reqs, &mut scratch, &mut outcomes);
}
