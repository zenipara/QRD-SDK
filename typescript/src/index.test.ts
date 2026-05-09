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
});