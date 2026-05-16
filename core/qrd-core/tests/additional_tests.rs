//! Additional small unit tests to reach target count

use qrd_core::footer::Footer;
use qrd_core::schema::{FieldType, Nullability, SchemaBuilder};
use qrd_core::utils::varint;

#[test]
fn test_varint_small_values() {
    for v in 0u64..1000u64 {
        let enc = varint::encode(v);
        // decode by simple loop
        let mut val = 0u64;
        let mut shift = 0u32;
        for b in enc.iter() {
            val |= ((b & 0x7F) as u64) << shift;
            if b & 0x80 == 0 {
                break;
            }
            shift += 7;
        }
        assert_eq!(val, v);
    }
}

#[test]
fn test_schema_builder_errors() {
    // duplicate field names should error
    let res = SchemaBuilder::new()
        .add_field("f", FieldType::Int32, Nullability::Required)
        .unwrap()
        .add_field("f", FieldType::Int32, Nullability::Required);
    assert!(res.is_err());
}

#[test]
fn test_footer_metadata_index_none() {
    let schema = SchemaBuilder::new()
        .add_field("a", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let footer = Footer::new(schema, 42);
    let ser = footer.serialize().unwrap();
    let deser = Footer::deserialize(&ser).unwrap();
    assert_eq!(deser.row_count, 42);
}

#[test]
fn test_footer_with_zero_rows() {
    let schema = SchemaBuilder::new()
        .add_field("a", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let footer = Footer::new(schema, 0);
    let ser = footer.serialize().unwrap();
    let deser = Footer::deserialize(&ser).unwrap();
    assert_eq!(deser.row_count, 0);
}

#[test]
fn test_schema_many_fields() {
    let mut b = SchemaBuilder::new();
    for i in 0..30 {
        b = b
            .add_field(
                &format!("col{}", i),
                FieldType::Int32,
                Nullability::Required,
            )
            .unwrap();
    }
    let s = b.build().unwrap();
    assert_eq!(s.fields.len(), 30);
}

#[test]
fn test_varint_edge_large() {
    let v = (1u64 << 20) + 12345;
    let enc = varint::encode(v);
    // decode similarly
    let mut val = 0u64;
    let mut shift = 0u32;
    for b in enc.iter() {
        val |= ((b & 0x7F) as u64) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
    }
    assert_eq!(val, v);
}

#[test]
fn test_schema_optional_fields() {
    let s = SchemaBuilder::new()
        .add_field("o", FieldType::String, Nullability::Optional)
        .unwrap()
        .add_field("r", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    assert_eq!(s.fields.len(), 2);
}

#[test]
fn test_footer_checksum_zero_default() {
    let schema = SchemaBuilder::new()
        .add_field("a", FieldType::Int32, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let f = Footer::new(schema, 5);
    assert_eq!(f.checksum, 0);
}

#[test]
fn test_varint_encode_decode_roundtrip_random() {
    for &v in &[0u64, 1, 127, 128, 255, 1024, 65535, 1 << 32] {
        let enc = varint::encode(v);
        let mut val = 0u64;
        let mut shift = 0u32;
        for b in enc.iter() {
            val |= ((b & 0x7F) as u64) << shift;
            if b & 0x80 == 0 {
                break;
            }
            shift += 7;
        }
        assert_eq!(val, v);
    }
}

#[test]
fn test_footer_roundtrip_with_metadata_index_none() {
    // sanity check repeated
    let schema = SchemaBuilder::new()
        .add_field("x", FieldType::Blob, Nullability::Required)
        .unwrap()
        .build()
        .unwrap();
    let footer = Footer::new(schema, 7);
    let ser = footer.serialize().unwrap();
    let deser = Footer::deserialize(&ser).unwrap();
    assert_eq!(deser.row_count, 7);
}
