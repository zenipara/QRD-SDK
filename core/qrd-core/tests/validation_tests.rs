//! Validation tests for QRD Core
//! Focus on Validator, CorruptionDetector, and footer validation

use qrd_core::validation::{Validator, CorruptionDetector, CorruptionType};
use qrd_core::footer::Footer;
use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};

#[test]
fn test_validate_magic_ok() {
    let magic = *qrd_core::QRD_MAGIC;
    assert!(Validator::validate_magic(&magic).is_ok());
}

#[test]
fn test_validate_magic_bad() {
    let bad_magic = [b'X', b'R', b'D', 0x01];
    assert!(Validator::validate_magic(&bad_magic).is_err());
}

#[test]
fn test_corruption_detect_truncation() {
    let report = CorruptionDetector::detect_truncation(10, 100).unwrap();
    assert_eq!(report.corruption_type, CorruptionType::TruncatedFile);
}

#[test]
fn test_corruption_row_count_mismatch() {
    let report = CorruptionDetector::detect_row_count_mismatch(5, 10).unwrap();
    assert_eq!(report.corruption_type, CorruptionType::RowCountMismatch);
}

#[test]
fn test_footer_serialization_roundtrip() {
    let schema = SchemaBuilder::new()
        .add_field("id", FieldType::Int64, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();

    let footer = Footer::new(schema, 1234);
    let bytes = footer.serialize().unwrap();
    let parsed = Footer::deserialize(&bytes).unwrap();
    assert_eq!(parsed.row_count, 1234);
}

#[test]
fn test_detect_invalid_offset_error() {
    let err = CorruptionDetector::detect_invalid_offset(500, 100).unwrap();
    assert_eq!(err.corruption_type, CorruptionType::InvalidOffset);
}

#[test]
fn test_validate_monotonic_offsets_ok() {
    let offsets = vec![10u64, 20, 30, 100];
    assert!(CorruptionDetector::validate_monotonic_offsets(&offsets).is_ok());
}

#[test]
fn test_validate_monotonic_offsets_fail() {
    let offsets = vec![10u64, 5, 20];
    assert!(CorruptionDetector::validate_monotonic_offsets(&offsets).is_err());
}
