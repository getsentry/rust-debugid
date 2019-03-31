use debugid::CodeId;
use uuid::Uuid;

#[test]
fn test_new() {
    let id = CodeId::new("dfb8e43af2423d73a453aeb6a777ef75".into());
    assert_eq!(id.as_str(), "dfb8e43af2423d73a453aeb6a777ef75");
    assert_eq!(
        id.as_bytes(),
        Some(&b"\xdf\xb8\xe4\x3a\xf2\x42\x3d\x73\xa4\x53\xae\xb6\xa7\x77\xef\x75"[..])
    );
    assert_eq!(
        id.uuid(),
        Some(Uuid::parse_str("dfb8e43a-f242-3d73-a453-aeb6a777ef75").unwrap())
    );
}

#[test]
fn test_case_sensitive() {
    let id = CodeId::new("dfb8e43af2423d73a453aeb6a777ef75FFFFFF".into());
    assert_eq!(id.as_str(), "dfb8e43af2423d73a453aeb6a777ef75FFFFFF");
    // on the other hand, bytes ignore capitalization
    assert_eq!(
        id.as_bytes(),
        Some(&b"\xdf\xb8\xe4\x3a\xf2\x42\x3d\x73\xa4\x53\xae\xb6\xa7\x77\xef\x75\xff\xff\xff"[..])
    );
}

#[test]
fn test_from_binary() {
    let binary = b"\xdf\xb8\xe4\x3a\xf2\x42\x3d\x73\xa4\x53\xae\xb6\xa7\x77\xef\x75";
    let id = CodeId::from_binary(&binary[..]);
    assert_eq!(id.as_str(), "dfb8e43af2423d73a453aeb6a777ef75");
    assert_eq!(id.as_bytes(), Some(&binary[..]));
}

#[test]
fn test_is_nil() {
    let id = CodeId::new(String::new());
    assert!(id.is_nil());
}
