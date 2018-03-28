extern crate debugid;
extern crate uuid;
use std::str::FromStr;

use uuid::Uuid;
use debugid::DebugId;

#[test]
fn test_parse_zero() {
    assert_eq!(
        DebugId::from_str("dfb8e43a-f242-3d73-a453-aeb6a777ef75").unwrap(),
        DebugId::from_parts(
            Uuid::parse_str("dfb8e43a-f242-3d73-a453-aeb6a777ef75").unwrap(),
            0,
        )
    );
}

#[test]
fn test_parse_short() {
    assert_eq!(
        DebugId::from_str("dfb8e43a-f242-3d73-a453-aeb6a777ef75-a").unwrap(),
        DebugId::from_parts(
            Uuid::parse_str("dfb8e43a-f242-3d73-a453-aeb6a777ef75").unwrap(),
            10,
        )
    );
}

#[test]
fn test_parse_long() {
    assert_eq!(
        DebugId::from_str("dfb8e43a-f242-3d73-a453-aeb6a777ef75-feedface").unwrap(),
        DebugId::from_parts(
            Uuid::parse_str("dfb8e43a-f242-3d73-a453-aeb6a777ef75").unwrap(),
            4277009102,
        )
    );
}

#[test]
fn test_parse_compact() {
    assert_eq!(
        DebugId::from_str("dfb8e43af2423d73a453aeb6a777ef75feedface").unwrap(),
        DebugId::from_parts(
            Uuid::parse_str("dfb8e43a-f242-3d73-a453-aeb6a777ef75").unwrap(),
            4277009102,
        )
    );
}

#[test]
fn test_parse_upper() {
    assert_eq!(
        DebugId::from_str("DFB8E43A-F242-3D73-A453-AEB6A777EF75-FEEDFACE").unwrap(),
        DebugId::from_parts(
            Uuid::parse_str("dfb8e43a-f242-3d73-a453-aeb6a777ef75").unwrap(),
            4277009102,
        )
    );
}

#[test]
fn test_to_string_zero() {
    let id = DebugId::from_parts(
        Uuid::parse_str("dfb8e43a-f242-3d73-a453-aeb6a777ef75").unwrap(),
        0,
    );

    assert_eq!(id.to_string(), "dfb8e43a-f242-3d73-a453-aeb6a777ef75");
}

#[test]
fn test_to_string_short() {
    let id = DebugId::from_parts(
        Uuid::parse_str("dfb8e43a-f242-3d73-a453-aeb6a777ef75").unwrap(),
        10,
    );

    assert_eq!(id.to_string(), "dfb8e43a-f242-3d73-a453-aeb6a777ef75-a");
}

#[test]
fn test_to_string_long() {
    let id = DebugId::from_parts(
        Uuid::parse_str("dfb8e43a-f242-3d73-a453-aeb6a777ef75").unwrap(),
        4277009102,
    );

    assert_eq!(
        id.to_string(),
        "dfb8e43a-f242-3d73-a453-aeb6a777ef75-feedface"
    );
}

#[test]
fn test_parse_error_short() {
    assert!(DebugId::from_str("dfb8e43a-f242-3d73-a453-aeb6a777ef7").is_err());
}

#[test]
fn test_parse_error_long() {
    assert!(DebugId::from_str("dfb8e43a-f242-3d73-a453-aeb6a777ef75-feedface1").is_err())
}

#[test]
fn test_parse_breakpad_zero() {
    assert_eq!(
        DebugId::from_breakpad("DFB8E43AF2423D73A453AEB6A777EF750").unwrap(),
        DebugId::from_parts(
            Uuid::parse_str("DFB8E43AF2423D73A453AEB6A777EF75").unwrap(),
            0,
        )
    );
}

#[test]
fn test_parse_breakpad_short() {
    assert_eq!(
        DebugId::from_breakpad("DFB8E43AF2423D73A453AEB6A777EF75a").unwrap(),
        DebugId::from_parts(
            Uuid::parse_str("DFB8E43AF2423D73A453AEB6A777EF75").unwrap(),
            10,
        )
    );
}

#[test]
fn test_parse_breakpad_long() {
    assert_eq!(
        DebugId::from_breakpad("DFB8E43AF2423D73A453AEB6A777EF75feedface").unwrap(),
        DebugId::from_parts(
            Uuid::parse_str("DFB8E43AF2423D73A453AEB6A777EF75").unwrap(),
            4277009102,
        )
    );
}

#[test]
fn test_to_string_breakpad_zero() {
    let id = DebugId::from_parts(
        Uuid::parse_str("DFB8E43AF2423D73A453AEB6A777EF75").unwrap(),
        0,
    );

    assert_eq!(
        id.breakpad().to_string(),
        "DFB8E43AF2423D73A453AEB6A777EF750"
    );
}

#[test]
fn test_to_string_breakpad_short() {
    let id = DebugId::from_parts(
        Uuid::parse_str("DFB8E43AF2423D73A453AEB6A777EF75").unwrap(),
        10,
    );

    assert_eq!(
        id.breakpad().to_string(),
        "DFB8E43AF2423D73A453AEB6A777EF75a"
    );
}

#[test]
fn test_to_string_breakpad_long() {
    let id = DebugId::from_parts(
        Uuid::parse_str("DFB8E43AF2423D73A453AEB6A777EF75").unwrap(),
        4277009102,
    );

    assert_eq!(
        id.breakpad().to_string(),
        "DFB8E43AF2423D73A453AEB6A777EF75feedface"
    );
}

#[test]
fn test_parse_breakpad_error_short() {
    assert!(DebugId::from_breakpad("DFB8E43AF2423D73A453AEB6A777EF7").is_err());
}

#[test]
fn test_parse_breakpad_error_long() {
    assert!(DebugId::from_breakpad("DFB8E43AF2423D73A453AEB6A777EF75feedface1").is_err())
}

#[test]
fn test_debug_id_debug() {
    let id = DebugId::from_parts(
        Uuid::parse_str("DFB8E43AF2423D73A453AEB6A777EF75").unwrap(),
        10,
    );

    assert_eq!(
        format!("{:?}", id),
        "DebugId { uuid: Uuid(\"\
         dfb8e43a-f242-3d73-a453-aeb6a777ef75\"), appendix: 10 }"
    );
}
