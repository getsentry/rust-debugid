#![cfg(feature = "serde")]

use debugid::DebugId;
use uuid::Uuid;

#[test]
fn test_deserialize() {
    assert_eq!(
        DebugId::from_parts(
            Uuid::parse_str("dfb8e43a-f242-3d73-a453-aeb6a777ef75").unwrap(),
            0,
        ),
        serde_json::from_str("\"dfb8e43a-f242-3d73-a453-aeb6a777ef75\"").unwrap(),
    );
}

#[test]
fn test_serialize() {
    let id = DebugId::from_parts(
        Uuid::parse_str("dfb8e43a-f242-3d73-a453-aeb6a777ef75").unwrap(),
        0,
    );

    assert_eq!(
        "\"dfb8e43a-f242-3d73-a453-aeb6a777ef75\"",
        serde_json::to_string(&id).unwrap(),
    );
}
