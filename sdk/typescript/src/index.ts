// TypeScript bindings for QRD SDK using WebAssembly

import * as wasm from 'qrd-wasm';

export enum FieldType {
  BOOLEAN = "BOOLEAN",
  INT8 = "INT8",
  INT16 = "INT16",
  INT32 = "INT32",
  INT64 = "INT64",
  UINT8 = "UINT8",
  UINT16 = "UINT16",
  UINT32 = "UINT32",
  UINT64 = "UINT64",
  FLOAT32 = "FLOAT32",
  FLOAT64 = "FLOAT64",
  TIMESTAMP = "TIMESTAMP",
  DATE = "DATE",
  TIME = "TIME",
  DURATION = "DURATION",
  STRING = "STRING",
  ENUM = "ENUM",
  UUID = "UUID",
  BLOB = "BLOB",
  DECIMAL = "DECIMAL",
}

export enum Nullability {
  REQUIRED = "REQUIRED",
  OPTIONAL = "OPTIONAL",
  REPEATED = "REPEATED",
}

export interface Field {
  name: string;
  fieldType: FieldType;
  nullability: Nullability;
  metadata?: string;
}

export interface Schema {
  fields: Field[];
  schemaId: number;
}

export class SchemaBuilder {
  private wasmBuilder: wasm.SchemaBuilder;

  constructor() {
    this.wasmBuilder = new wasm.SchemaBuilder();
  }

  addField(name: string, fieldType: FieldType, nullability: Nullability, metadata?: string): this {
    const field = new wasm.SchemaField(name, fieldType, nullability === Nullability.OPTIONAL, nullability === Nullability.REPEATED);
    if (metadata) {
      field.set_metadata(metadata);
    }
    this.wasmBuilder.add_field(field);
    return this;
  }

  build(): Schema {
    const wasmSchema = this.wasmBuilder.build();
    const fields: Field[] = [];
    const fieldNames = wasmSchema.field_names();

    for (let i = 0; i < fieldNames.length; i++) {
      const name = fieldNames[i] as string;
      // Note: In a full implementation, we'd need to expose more field details from WASM
      fields.push({
        name,
        fieldType: FieldType.STRING, // Placeholder - would need proper field type exposure
        nullability: Nullability.REQUIRED, // Placeholder
      });
    }

    return {
      fields,
      schemaId: wasmSchema.schema_id(),
    };
  }
}

export class FileWriter {
  private wasmWriter: wasm.FileWriter;

  constructor(schema: Schema) {
    const builder = new SchemaBuilder();
    for (const field of schema.fields) {
      builder.addField(field.name, field.fieldType, field.nullability, field.metadata);
    }
    const wasmSchema = builder.wasmBuilder.build();
    this.wasmWriter = new wasm.FileWriter(wasmSchema);
  }

  writeRow(row: any[]): void {
    // Convert row to JS object for WASM
    const rowObj: any = {};
    // This is a simplified mapping - in practice you'd need proper field mapping
    for (let i = 0; i < row.length; i++) {
      rowObj[`field_${i}`] = row[i];
    }
    this.wasmWriter.write_row(rowObj);
  }

  finish(): Uint8Array {
    return this.wasmWriter.finish();
  }
}

export class FileReader {
  private wasmReader: wasm.FileReader;

  constructor(data: Uint8Array) {
    this.wasmReader = new wasm.FileReader(data);
  }

  getSchema(): Schema {
    const wasmSchema = this.wasmReader.schema();
    const fields: Field[] = [];
    const fieldNames = wasmSchema.field_names();

    for (let i = 0; i < fieldNames.length; i++) {
      const name = fieldNames[i] as string;
      fields.push({
        name,
        fieldType: FieldType.STRING, // Placeholder
        nullability: Nullability.REQUIRED, // Placeholder
      });
    }

    return {
      fields,
      schemaId: wasmSchema.schema_id(),
    };
  }

  getRowCount(): number {
    return this.wasmReader.row_count();
  }

  readRow(): any[] | null {
    const row = this.wasmReader.read_row();
    if (row === null) {
      return null;
    }
    // Convert JS object back to array
    // This is simplified - in practice you'd map based on schema
    return Object.values(row);
  }

  readAllRows(): any[][] {
    const rows: any[][] = [];
    let row: any[] | null;
    while ((row = this.readRow()) !== null) {
      rows.push(row);
    }
    return rows;
  }
}

// Utility functions for data serialization
export class Serializer {
  static serializeInt64(value: number): Uint8Array {
    const buffer = new ArrayBuffer(8);
    const view = new DataView(buffer);
    view.setBigInt64(0, BigInt(value), true); // true for little-endian
    return new Uint8Array(buffer);
  }

  static serializeFloat64(value: number): Uint8Array {
    const buffer = new ArrayBuffer(8);
    const view = new DataView(buffer);
    view.setFloat64(0, value, true); // true for little-endian
    return new Uint8Array(buffer);
  }

  static serializeString(value: string): Uint8Array {
    const encoder = new TextEncoder();
    const bytes = encoder.encode(value);
    const buffer = new ArrayBuffer(4 + bytes.length);
    const view = new DataView(buffer);

    // Write length prefix
    view.setUint32(0, bytes.length, true); // true for little-endian

    // Write string bytes
    const uint8View = new Uint8Array(buffer);
    uint8View.set(bytes, 4);

    return uint8View;
  }

  static deserializeInt64(bytes: Uint8Array): number {
    const view = new DataView(bytes.buffer, bytes.byteOffset, 8);
    return Number(view.getBigInt64(0, true)); // true for little-endian
  }

  static deserializeFloat64(bytes: Uint8Array): number {
    const view = new DataView(bytes.buffer, bytes.byteOffset, 8);
    return view.getFloat64(0, true); // true for little-endian
  }

  static deserializeString(bytes: Uint8Array): string {
    const view = new DataView(bytes.buffer, bytes.byteOffset, 4);
    const length = view.getUint32(0, true); // true for little-endian

    const decoder = new TextDecoder();
    return decoder.decode(bytes.subarray(4, 4 + length));
  }
}