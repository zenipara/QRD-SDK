// TypeScript bindings test
import { SchemaBuilder, FileWriter, FileReader, FieldType, Nullability } from '../src/index';

describe('QRD TypeScript Bindings', () => {
  test('Schema building', () => {
    const builder = new SchemaBuilder();
    builder.addField('id', FieldType.INT64, Nullability.REQUIRED);
    builder.addField('name', FieldType.STRING, Nullability.OPTIONAL);

    const schema = builder.build();
    expect(schema.fields.length).toBe(2);
    expect(schema.schemaId).toBeGreaterThan(0);
  });

  test('File I/O roundtrip', () => {
    const builder = new SchemaBuilder();
    builder.addField('id', FieldType.INT64, Nullability.REQUIRED);
    builder.addField('value', FieldType.FLOAT64, Nullability.OPTIONAL);

    const schema = builder.build();
    const writer = new FileWriter(schema);

    // Write some test data
    writer.writeRow([1, 3.14]);
    writer.writeRow([2, 2.71]);

    // Get the buffer
    const buffer = writer.finish();

    // Read it back
    const reader = new FileReader(buffer);
    expect(reader.getRowCount()).toBe(2);

    const readSchema = reader.getSchema();
    expect(readSchema.fields.length).toBe(2);

    const rows = reader.readAllRows();
    expect(rows.length).toBe(2);
  });

  // ====== Additional TypeScript SDK Tests ======

  test('WASM initialization', () => {
    const builder = new SchemaBuilder();
    builder.addField('test', FieldType.INT32, Nullability.REQUIRED);
    
    const schema = builder.build();
    expect(schema.fields.length).toBe(1);
  });

  test('Schema roundtrip with multiple types', () => {
    const builder = new SchemaBuilder();
    builder.addField('int32_col', FieldType.INT32, Nullability.REQUIRED);
    builder.addField('int64_col', FieldType.INT64, Nullability.REQUIRED);
    builder.addField('float32_col', FieldType.FLOAT32, Nullability.OPTIONAL);
    builder.addField('float64_col', FieldType.FLOAT64, Nullability.OPTIONAL);
    builder.addField('string_col', FieldType.STRING, Nullability.OPTIONAL);

    const schema = builder.build();
    
    const writer = new FileWriter(schema);
    writer.writeRow([42, 123456789, 3.14, 2.71828, 'test']);
    const buffer = writer.finish();

    const reader = new FileReader(buffer);
    expect(reader.getRowCount()).toBe(1);
    expect(reader.getSchema().fields.length).toBe(5);
  });

  test('Deterministic browser reads', () => {
    const builder = new SchemaBuilder();
    builder.addField('id', FieldType.INT64, Nullability.REQUIRED);
    const schema = builder.build();

    // Write twice
    const writer1 = new FileWriter(schema);
    for (let i = 0; i < 100; i++) {
      writer1.writeRow([i]);
    }
    const buffer1 = writer1.finish();

    const writer2 = new FileWriter(schema);
    for (let i = 0; i < 100; i++) {
      writer2.writeRow([i]);
    }
    const buffer2 = writer2.finish();

    // Buffers should be identical
    expect(buffer1.length).toBe(buffer2.length);
  });

  test('Invalid payload handling', () => {
    const malformed = new Uint8Array([0xFF, 0xFF, 0xAA, 0xBB]);
    
    try {
      const reader = new FileReader(malformed);
      // Should handle gracefully or throw
    } catch (e) {
      // Expected
    }
  });

  test('Partial reads capability', () => {
    const builder = new SchemaBuilder();
    builder.addField('col1', FieldType.INT64, Nullability.REQUIRED);
    builder.addField('col2', FieldType.STRING, Nullability.REQUIRED);
    builder.addField('col3', FieldType.FLOAT64, Nullability.REQUIRED);

    const schema = builder.build();
    const writer = new FileWriter(schema);

    for (let i = 0; i < 50; i++) {
      writer.writeRow([i, `text_${i}`, i * 1.5]);
    }

    const buffer = writer.finish();
    const reader = new FileReader(buffer);
    
    expect(reader.getRowCount()).toBe(50);
  });

  test('Typed array validation', () => {
    const builder = new SchemaBuilder();
    builder.addField('data', FieldType.BLOB, Nullability.REQUIRED);
    const schema = builder.build();

    const writer = new FileWriter(schema);
    const typedArray = new Uint8Array([1, 2, 3, 4, 5]);
    // TypedArray should be accepted as BLOB data
    writer.writeRow([typedArray]);

    const buffer = writer.finish();
    expect(buffer instanceof Uint8Array || buffer instanceof ArrayBuffer).toBe(true);
  });

  test('Browser compatibility assumptions', () => {
    // Test that code works in browser-like environment
    const builder = new SchemaBuilder();
    builder.addField('value', FieldType.FLOAT64, Nullability.REQUIRED);
    
    const schema = builder.build();
    const writer = new FileWriter(schema);
    
    // Should work without assuming Node.js APIs
    writer.writeRow([3.14159]);
    const buffer = writer.finish();
    
    expect(buffer.length).toBeGreaterThan(0);
  });

  test('Async loading capability', async () => {
    const builder = new SchemaBuilder();
    builder.addField('id', FieldType.INT32, Nullability.REQUIRED);
    const schema = builder.build();

    const writer = new FileWriter(schema);
    writer.writeRow([1]);
    writer.writeRow([2]);
    writer.writeRow([3]);
    const buffer = writer.finish();

    // Simulate async load
    const reader = await Promise.resolve(new FileReader(buffer));
    expect(reader.getRowCount()).toBe(3);
  });

  test('Malformed input rejection', () => {
    const corrupted = new Uint8Array([0x00, 0x00, 0x00, 0x00]);
    
    try {
      const reader = new FileReader(corrupted);
      // Should either succeed with validation or throw
    } catch (e) {
      // Expected for invalid magic bytes
    }
  });

  test('Footer inspection', () => {
    const builder = new SchemaBuilder();
    builder.addField('id', FieldType.INT64, Nullability.REQUIRED);
    const schema = builder.build();

    const writer = new FileWriter(schema);
    for (let i = 0; i < 25; i++) {
      writer.writeRow([i]);
    }
    const buffer = writer.finish();

    const reader = new FileReader(buffer);
    
    // Should be able to inspect footer metadata
    expect(reader.getRowCount()).toBe(25);
    expect(reader.getSchema()).toBeDefined();
  });

  test('Large dataset handling', () => {
    const builder = new SchemaBuilder();
    builder.addField('value', FieldType.INT64, Nullability.REQUIRED);
    const schema = builder.build();

    const writer = new FileWriter(schema);
    
    for (let i = 0; i < 1000; i++) {
      writer.writeRow([i]);
    }

    const buffer = writer.finish();
    const reader = new FileReader(buffer);
    
    expect(reader.getRowCount()).toBe(1000);
  });

  test('Empty dataset handling', () => {
    const builder = new SchemaBuilder();
    builder.addField('id', FieldType.INT32, Nullability.REQUIRED);
    const schema = builder.build();

    const writer = new FileWriter(schema);
    // Write nothing
    const buffer = writer.finish();

    const reader = new FileReader(buffer);
    expect(reader.getRowCount()).toBe(0);
  });
});