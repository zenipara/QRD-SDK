"""Test suite for QRD Python binding"""
import struct
import tempfile
import os


def test_basic_write_read():
    """Test basic schema creation and file I/O"""
    import qrd

    # Create schema
    schema_builder = qrd.SchemaBuilder()
    schema_builder.add_field("id", "INT64", required=True)
    schema_builder.add_field("name", "STRING", required=False)
    schema = schema_builder.build()

    # Write data
    with tempfile.NamedTemporaryFile(delete=False, suffix=".qrd") as tmp:
        tmp_path = tmp.name

    try:
        with qrd.Writer(tmp_path, schema) as writer:
            for i in range(100):
                # i64 LE for id
                id_bytes = struct.pack("<q", i)
                # length-prefixed string for name
                name_str = f"user_{i}".encode()
                name_bytes = struct.pack("<I", len(name_str)) + name_str
                writer.write_row([id_bytes, name_bytes])

        # Read data back
        reader = qrd.Reader(tmp_path)
        assert reader.row_count() == 100

        # Verify schema
        read_schema = reader.schema()
        assert read_schema.field_count() == 2

        print("✓ test_basic_write_read passed")
    finally:
        if os.path.exists(tmp_path):
            os.remove(tmp_path)


def test_schema_types():
    """Test schema builder with various field types"""
    import qrd

    schema_builder = qrd.SchemaBuilder()
    schema_builder.add_field("col_bool", "BOOLEAN", required=True)
    schema_builder.add_field("col_int32", "INT32", required=True)
    schema_builder.add_field("col_int64", "INT64", required=True)
    schema_builder.add_field("col_float", "FLOAT64", required=False)
    schema_builder.add_field("col_string", "STRING", required=False)
    
    schema = schema_builder.build()
    assert schema.field_count() == 5

    print("✓ test_schema_types passed")


if __name__ == "__main__":
    test_schema_types()
    test_basic_write_read()
    print("\n✅ All Python binding tests passed!")
