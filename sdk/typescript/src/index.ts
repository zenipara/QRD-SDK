import init, { QrdSchemaBuilder, QrdMemWriter } from '../pkg/qrd_wasm';

export async function createQrdFile(
  fields: Array<{ name: string; type: string }>,
  rows: Array<Array<Uint8Array>>
): Promise<Uint8Array> {
  await init();

  const builder = new QrdSchemaBuilder();
  for (const field of fields) {
    builder.add_field(field.name, field.type);
  }
  const schema = builder.build();

  const writer = new QrdMemWriter(schema);
  for (const row of rows) {
    writer.write_row(row);
  }

  return writer.finish();
}

export async function readQrdFile(data: Uint8Array): Promise<{
  rowCount: number;
}> {
  await init();
  // Reader implementation would follow similar pattern
  return {
    rowCount: 0,
  };
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