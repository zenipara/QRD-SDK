export enum FieldType {
  BOOLEAN = 'BOOLEAN',
  INT8 = 'INT8',
  INT16 = 'INT16',
  INT32 = 'INT32',
  INT64 = 'INT64',
  UINT8 = 'UINT8',
  UINT16 = 'UINT16',
  UINT32 = 'UINT32',
  UINT64 = 'UINT64',
  FLOAT32 = 'FLOAT32',
  FLOAT64 = 'FLOAT64',
  TIMESTAMP = 'TIMESTAMP',
  DATE = 'DATE',
  TIME = 'TIME',
  DURATION = 'DURATION',
  STRING = 'STRING',
  ENUM = 'ENUM',
  UUID = 'UUID',
  BLOB = 'BLOB',
  DECIMAL = 'DECIMAL',
}

export enum Nullability {
  REQUIRED = 'REQUIRED',
  OPTIONAL = 'OPTIONAL',
}

export interface SchemaField {
  name: string;
  type: FieldType;
  required: boolean;
}

export class Schema {
  public readonly fields: SchemaField[];
  public readonly schemaId: number;

  constructor(fields: SchemaField[]) {
    this.fields = fields.map((field) => ({ ...field }));
    this.schemaId = computeSchemaId(this.fields);
  }

  fieldCount(): number {
    return this.fields.length;
  }
}

export class SchemaBuilder {
  private fields: SchemaField[] = [];

  addField(name: string, type: FieldType, nullability: Nullability): this {
    this.fields.push({
      name,
      type,
      required: nullability === Nullability.REQUIRED,
    });
    return this;
  }

  build(): Schema {
    return new Schema(this.fields);
  }
}

export class FileWriter {
  private readonly schema: Schema;
  private rows: Array<Array<number | string | Uint8Array | null>> = [];
  private finished = false;

  constructor(schema: Schema) {
    this.schema = schema;
  }

  writeRow(row: Array<number | string | Uint8Array | null>): void {
    if (this.finished) {
      throw new Error('Writer already finished');
    }
    this.rows.push(row.map(cloneCell));
  }

  finish(): Uint8Array {
    if (this.finished) {
      throw new Error('Writer already finished');
    }

    this.finished = true;
    const payload = {
      schema: {
        fields: this.schema.fields,
        schemaId: this.schema.schemaId,
      },
      rows: this.rows.map((row) => row.map(serializeCellForStorage)),
    };

    return encodePayload(payload);
  }
}

export class FileReader {
  private readonly schema: Schema;
  private readonly rows: Array<Array<number | string | Uint8Array | null>>;
  private cursor = 0;

  constructor(data: Uint8Array) {
    const payload = decodePayload(data);
    this.schema = new Schema(payload.schema.fields);
    this.rows = payload.rows.map((row) => row.map(deserializeCellFromStorage));
  }

  getRowCount(): number {
    return this.rows.length;
  }

  getSchema(): Schema {
    return this.schema;
  }

  readRow(): Array<number | string | Uint8Array | null> | null {
    if (this.cursor >= this.rows.length) {
      return null;
    }
    const row = this.rows[this.cursor];
    this.cursor += 1;
    return row.map(cloneCell);
  }

  readAllRows(): Array<Array<number | string | Uint8Array | null>> {
    const rows: Array<Array<number | string | Uint8Array | null>> = [];
    let row: Array<number | string | Uint8Array | null> | null;
    while ((row = this.readRow()) !== null) {
      rows.push(row);
    }
    return rows;
  }
}

export async function createQrdFile(
  fields: Array<{ name: string; type: string }>,
  rows: Array<Array<number | string | Uint8Array | null>>,
): Promise<Uint8Array> {
  const builder = new SchemaBuilder();
  for (const field of fields) {
    builder.addField(field.name, parseFieldType(field.type), Nullability.REQUIRED);
  }
  const writer = new FileWriter(builder.build());
  for (const row of rows) {
    writer.writeRow(row);
  }
  return writer.finish();
}

export async function readQrdFile(data: Uint8Array): Promise<{ rowCount: number }> {
  const reader = new FileReader(data);
  return {
    rowCount: reader.getRowCount(),
  };
}

export class Serializer {
  static serializeInt64(value: number): Uint8Array {
    return writeInt64(value);
  }

  static serializeFloat64(value: number): Uint8Array {
    return writeFloat64(value);
  }

  static serializeString(value: string): Uint8Array {
    return toUtf8Bytes(value);
  }

  static deserializeInt64(bytes: Uint8Array): number {
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    return Number(view.getBigInt64(0, true));
  }

  static deserializeFloat64(bytes: Uint8Array): number {
    const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    return view.getFloat64(0, true);
  }

  static deserializeString(bytes: Uint8Array): string {
    return fromUtf8Bytes(bytes);
  }
}

function parseFieldType(type: string): FieldType {
  const normalized = type.toUpperCase();
  if (normalized in FieldType) {
    return normalized as FieldType;
  }
  throw new Error(`Unknown field type: ${type}`);
}

// TODO: Implement support for multiple row groups and row group sizing.
// TODO: Support full schema metadata entries in footer serialization.
// TODO: Add compression and alternative encodings beyond plain storage.
// TODO: Add partial read/inspect APIs to avoid full row-group decoding.
// TODO: Add support for repeated fields and nested/array types.
const TEXT_ENCODER = new TextEncoder();
const TEXT_DECODER = new TextDecoder('utf-8');

const FIELD_TYPE_IDS: Record<FieldType, number> = {
  BOOLEAN: 1,
  INT8: 2,
  INT16: 3,
  INT32: 4,
  INT64: 5,
  UINT8: 10,
  UINT16: 11,
  UINT32: 12,
  UINT64: 13,
  FLOAT32: 18,
  FLOAT64: 19,
  TIMESTAMP: 20,
  DATE: 21,
  TIME: 22,
  DURATION: 23,
  STRING: 24,
  ENUM: 25,
  UUID: 26,
  BLOB: 27,
  DECIMAL: 28,
};

function fieldTypeToId(type: FieldType): number {
  return FIELD_TYPE_IDS[type];
}

function fieldTypeFromId(id: number): FieldType {
  const entry = Object.entries(FIELD_TYPE_IDS).find(([, value]) => value === id);
  if (!entry) {
    throw new Error(`Unknown field type ID: ${id}`);
  }
  return entry[0] as FieldType;
}

function nullabilityToId(required: boolean): number {
  return required ? 0 : 1;
}

function idToNullability(id: number): boolean {
  if (id === 0) return true;
  if (id === 1) return false;
  throw new Error(`Unknown nullability ID: ${id}`);
}

function writeUint8(value: number): Uint8Array {
  return new Uint8Array([value & 0xff]);
}

function writeUint16(value: number): Uint8Array {
  const buffer = new ArrayBuffer(2);
  new DataView(buffer).setUint16(0, value, true);
  return new Uint8Array(buffer);
}

function writeUint32(value: number): Uint8Array {
  const buffer = new ArrayBuffer(4);
  new DataView(buffer).setUint32(0, value, true);
  return new Uint8Array(buffer);
}

function writeInt32(value: number): Uint8Array {
  const buffer = new ArrayBuffer(4);
  new DataView(buffer).setInt32(0, value, true);
  return new Uint8Array(buffer);
}

function writeInt64(value: number): Uint8Array {
  const buffer = new ArrayBuffer(8);
  new DataView(buffer).setBigInt64(0, BigInt(value), true);
  return new Uint8Array(buffer);
}

function writeUint64(value: number): Uint8Array {
  const buffer = new ArrayBuffer(8);
  new DataView(buffer).setBigUint64(0, BigInt(value), true);
  return new Uint8Array(buffer);
}

function writeFloat32(value: number): Uint8Array {
  const buffer = new ArrayBuffer(4);
  new DataView(buffer).setFloat32(0, value, true);
  return new Uint8Array(buffer);
}

function writeFloat64(value: number): Uint8Array {
  const buffer = new ArrayBuffer(8);
  new DataView(buffer).setFloat64(0, value, true);
  return new Uint8Array(buffer);
}

function concatUint8Arrays(chunks: Uint8Array[]): Uint8Array {
  const length = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
  const result = new Uint8Array(length);
  let offset = 0;
  for (const chunk of chunks) {
    result.set(chunk, offset);
    offset += chunk.length;
  }
  return result;
}

function toUtf8Bytes(value: string): Uint8Array {
  return TEXT_ENCODER.encode(value);
}

function fromUtf8Bytes(bytes: Uint8Array): string {
  return TEXT_DECODER.decode(bytes);
}

function computeSchemaId(fields: SchemaField[]): number {
  let hash = 2166136261;
  for (const field of fields) {
    const token = `${field.name}:${field.type}:${field.required ? '1' : '0'}`;
    for (let index = 0; index < token.length; index += 1) {
      hash ^= token.charCodeAt(index);
      hash = Math.imul(hash, 16777619);
    }
  }
  return hash >>> 0;
}

type StoredCell =
  | { kind: 'number'; value: number }
  | { kind: 'string'; value: string }
  | { kind: 'bytes'; value: number[] }
  | { kind: 'null' };

function cloneCell(value: number | string | Uint8Array | null): number | string | Uint8Array | null {
  if (value instanceof Uint8Array) {
    return new Uint8Array(value);
  }
  return value;
}

function serializeCellForStorage(value: number | string | Uint8Array | null): StoredCell {
  if (value === null) {
    return { kind: 'null' };
  }
  if (value instanceof Uint8Array) {
    return { kind: 'bytes', value: Array.from(value) };
  }
  if (typeof value === 'number') {
    return { kind: 'number', value };
  }
  return { kind: 'string', value };
}

function deserializeCellFromStorage(value: StoredCell): number | string | Uint8Array | null {
  if (value.kind === 'bytes') {
    return new Uint8Array(value.value);
  }
  if (value.kind === 'null') {
    return null;
  }
  return value.value;
}

let CRC32_TABLE: number[] | null = null;

function crc32(data: Uint8Array): number {
  if (CRC32_TABLE === null) {
    CRC32_TABLE = makeCrcTable();
  }
  let crc = 0xffffffff;
  for (let i = 0; i < data.length; i += 1) {
    crc = (crc >>> 8) ^ CRC32_TABLE[(crc ^ data[i]) & 0xff];
  }
  return (crc ^ 0xffffffff) >>> 0;
}

function makeCrcTable(): number[] {
  const table = new Array<number>(256);
  for (let i = 0; i < 256; i += 1) {
    let c = i;
    for (let j = 0; j < 8; j += 1) {
      if (c & 1) {
        c = 0xedb88320 ^ (c >>> 1);
      } else {
        c = c >>> 1;
      }
    }
    table[i] = c >>> 0;
  }
  return table;
}

function serializeSchema(fields: SchemaField[]): Uint8Array {
  const chunks: Uint8Array[] = [];
  chunks.push(writeUint16(1));
  chunks.push(writeUint16(fields.length));

  for (const field of fields) {
    const nameBytes = toUtf8Bytes(field.name);
    chunks.push(writeUint16(nameBytes.length));
    chunks.push(nameBytes);
    chunks.push(writeUint8(fieldTypeToId(field.type)));
    chunks.push(writeUint8(nullabilityToId(field.required)));
    chunks.push(writeUint16(0));
  }

  return concatUint8Arrays(chunks);
}

function deserializeSchema(data: Uint8Array): { schema: { fields: SchemaField[]; schemaId: number }; bytesRead: number } {
  if (data.length < 4) {
    throw new Error('Schema data too short');
  }

  let pos = 0;
  const version = new DataView(data.buffer, data.byteOffset, data.byteLength).getUint16(pos, true);
  pos += 2;
  if (version !== 1) {
    throw new Error(`Unsupported schema version: ${version}`);
  }

  const fieldCount = new DataView(data.buffer, data.byteOffset, data.byteLength).getUint16(pos, true);
  pos += 2;

  const fields: SchemaField[] = [];
  for (let i = 0; i < fieldCount; i += 1) {
    if (pos + 2 > data.length) {
      throw new Error('Unexpected end of schema data');
    }

    const nameLength = new DataView(data.buffer, data.byteOffset, data.byteLength).getUint16(pos, true);
    pos += 2;
    if (pos + nameLength + 4 > data.length) {
      throw new Error('Unexpected end of schema data');
    }

    const name = fromUtf8Bytes(data.subarray(pos, pos + nameLength));
    pos += nameLength;

    const typeId = data[pos];
    pos += 1;
    const fieldType = fieldTypeFromId(typeId);

    const nullabilityId = data[pos];
    pos += 1;
    const required = idToNullability(nullabilityId);

    const metadataCount = new DataView(data.buffer, data.byteOffset, data.byteLength).getUint16(pos, true);
    pos += 2;
    for (let j = 0; j < metadataCount; j += 1) {
      const keyLength = new DataView(data.buffer, data.byteOffset, data.byteLength).getUint16(pos, true);
      pos += 2;
      pos += keyLength;
      const valueLength = new DataView(data.buffer, data.byteOffset, data.byteLength).getUint16(pos, true);
      pos += 2;
      pos += valueLength;
    }

    fields.push({ name, type: fieldType, required });
  }

  const schemaId = computeSchemaId(fields);
  return { schema: { fields, schemaId }, bytesRead: pos };
}

function serializeValue(type: FieldType, value: number | string | Uint8Array): Uint8Array {
  switch (type) {
    case FieldType.BOOLEAN: {
      const boolValue = typeof value === 'number' ? value !== 0 : value === true;
      return writeUint8(boolValue ? 1 : 0);
    }
    case FieldType.INT8:
    case FieldType.UINT8: {
      return writeUint8(Number(value));
    }
    case FieldType.INT16:
      return writeInt32(Number(value)).subarray(0, 2);
    case FieldType.UINT16: {
      const buffer = new ArrayBuffer(2);
      new DataView(buffer).setUint16(0, Number(value), true);
      return new Uint8Array(buffer);
    }
    case FieldType.INT32:
      return writeInt32(Number(value));
    case FieldType.UINT32:
      return writeUint32(Number(value));
    case FieldType.INT64:
    case FieldType.TIMESTAMP:
    case FieldType.TIME:
    case FieldType.DURATION:
      return writeInt64(Number(value));
    case FieldType.UINT64:
      return writeUint64(Number(value));
    case FieldType.FLOAT32:
      return writeFloat32(Number(value));
    case FieldType.FLOAT64:
      return writeFloat64(Number(value));
    case FieldType.DATE:
      return writeInt32(Number(value));
    case FieldType.STRING: {
      const bytes = toUtf8Bytes(String(value));
      return concatUint8Arrays([writeUint32(bytes.length), bytes]);
    }
    case FieldType.BLOB: {
      const bytes = value instanceof Uint8Array ? value : new Uint8Array(Array.isArray(value) ? value : []);
      return concatUint8Arrays([writeUint32(bytes.length), bytes]);
    }
    default:
      throw new Error(`Unsupported field type for serialization: ${type}`);
  }
}

function deserializeValue(type: FieldType, data: Uint8Array, cursor: number): { value: number | string | Uint8Array; bytesRead: number } {
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  switch (type) {
    case FieldType.BOOLEAN:
      return { value: data[cursor] !== 0, bytesRead: 1 };
    case FieldType.INT8:
      return { value: view.getInt8(cursor), bytesRead: 1 };
    case FieldType.UINT8:
      return { value: view.getUint8(cursor), bytesRead: 1 };
    case FieldType.INT16:
      return { value: view.getInt16(cursor, true), bytesRead: 2 };
    case FieldType.UINT16:
      return { value: view.getUint16(cursor, true), bytesRead: 2 };
    case FieldType.INT32:
      return { value: view.getInt32(cursor, true), bytesRead: 4 };
    case FieldType.UINT32:
      return { value: view.getUint32(cursor, true), bytesRead: 4 };
    case FieldType.INT64:
    case FieldType.TIMESTAMP:
    case FieldType.TIME:
    case FieldType.DURATION:
      return { value: Number(view.getBigInt64(cursor, true)), bytesRead: 8 };
    case FieldType.UINT64:
      return { value: Number(view.getBigUint64(cursor, true)), bytesRead: 8 };
    case FieldType.FLOAT32:
      return { value: view.getFloat32(cursor, true), bytesRead: 4 };
    case FieldType.FLOAT64:
      return { value: view.getFloat64(cursor, true), bytesRead: 8 };
    case FieldType.DATE:
      return { value: view.getInt32(cursor, true), bytesRead: 4 };
    case FieldType.STRING: {
      const length = view.getUint32(cursor, true);
      const start = cursor + 4;
      const value = fromUtf8Bytes(data.subarray(start, start + length));
      return { value, bytesRead: 4 + length };
    }
    case FieldType.BLOB: {
      const length = view.getUint32(cursor, true);
      const start = cursor + 4;
      const bytes = data.subarray(start, start + length);
      return { value: new Uint8Array(bytes), bytesRead: 4 + length };
    }
    default:
      throw new Error(`Unsupported field type for deserialization: ${type}`);
  }
}

function packNullBitmap(nullMask: boolean[]): Uint8Array {
  const byteCount = Math.ceil(nullMask.length / 8);
  const result = new Uint8Array(byteCount);
  for (let i = 0; i < nullMask.length; i += 1) {
    if (nullMask[i]) {
      result[i >> 3] |= 1 << (i & 7);
    }
  }
  return result;
}

function unpackNullBitmap(data: Uint8Array, rowCount: number): boolean[] {
  const mask: boolean[] = [];
  for (let i = 0; i < rowCount; i += 1) {
    mask.push(((data[i >> 3] >> (i & 7)) & 1) !== 0);
  }
  return mask;
}

function serializeColumnChunk(
  field: SchemaField,
  fieldIndex: number,
  rows: Array<Array<number | string | Uint8Array | null>>,
): Uint8Array {
  const values: Uint8Array[] = [];
  const nullMask: boolean[] = [];
  let nullCount = 0;

  for (const row of rows) {
    const rawValue = row[fieldIndex];
    if (rawValue === undefined || rawValue === null) {
      if (field.required) {
        throw new Error(`Required field ${field.name} is missing a value`);
      }
      nullMask.push(false);
      nullCount += 1;
    } else {
      nullMask.push(true);
      values.push(serializeValue(field.type, rawValue as number | string | Uint8Array));
    }
  }

  const dataChunks: Uint8Array[] = [];
  if (!field.required) {
    dataChunks.push(packNullBitmap(nullMask));
  }
  dataChunks.push(...values);
  const data = concatUint8Arrays(dataChunks);
  const crc = crc32(data);

  return concatUint8Arrays([
    writeUint8(0),
    writeUint8(0),
    writeUint32(data.length),
    writeUint32(data.length),
    writeUint32(nullCount),
    writeUint32(0),
    data,
    writeUint32(crc),
  ]);
}

function serializeRowGroup(schema: Schema, rows: Array<Array<number | string | Uint8Array | null>>): Uint8Array {
  const rowCount = rows.length;
  const columnChunks: Uint8Array[] = [];

  for (let index = 0; index < schema.fields.length; index += 1) {
    const field = schema.fields[index];
    const chunk = serializeColumnChunk(field, index, rows);
    columnChunks.push(chunk);
  }

  const columnsBytes = concatUint8Arrays(columnChunks);
  const totalSize = columnsBytes.length;
  const compressedSize = columnsBytes.length;

  const rowGroupHeader = concatUint8Arrays([
    writeUint32(rowCount),
    writeUint32(totalSize),
    writeUint32(compressedSize),
    writeUint16(schema.fieldCount()),
    writeUint16(0),
  ]);

  const columnOffsets: Uint8Array[] = [];
  let offset = rowGroupHeader.length;
  for (const chunk of columnChunks) {
    columnOffsets.push(writeUint64(offset));
    offset += chunk.length;
  }

  const rowGroupMetadata = concatUint8Arrays([
    writeUint32(rowCount),
    writeUint32(0),
    writeUint32(crc32(columnsBytes)),
    concatUint8Arrays(columnOffsets),
  ]);

  return concatUint8Arrays([rowGroupHeader, columnsBytes, rowGroupMetadata]);
}

function serializeFooter(schema: Schema, rowGroupOffset: number, rowCount: number): Uint8Array {
  const schemaBytes = serializeSchema(schema.fields);
  const footerChunks: Uint8Array[] = [];
  footerChunks.push(schemaBytes);
  footerChunks.push(writeUint32(1));
  footerChunks.push(writeUint64(rowGroupOffset));
  footerChunks.push(writeUint8(0));
  footerChunks.push(writeUint32(0));
  const now = Math.floor(Date.now() / 1000);
  footerChunks.push(writeUint32(now));
  footerChunks.push(writeUint32(now));
  footerChunks.push(writeUint32(rowCount));

  const footerWithoutChecksum = concatUint8Arrays(footerChunks);
  const checksum = writeUint32(crc32(footerWithoutChecksum));

  return concatUint8Arrays([footerWithoutChecksum, checksum]);
}

function parseHeader(data: Uint8Array): { schemaId: number; rowCount: number; columnCount: number; rowGroupSize: number } {
  if (data.length < 32) {
    throw new Error('Data too short for QRD header');
  }
  if (toUtf8Bytes('QRD\x01').every((byte, index) => byte === data[index]) === false) {
    throw new Error('Invalid QRD magic bytes');
  }
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  const schemaId = view.getUint32(8, true);
  const rowCount = view.getUint32(16, true);
  const columnCount = view.getUint32(20, true);
  const rowGroupSize = view.getUint32(24, true);
  return { schemaId, rowCount, columnCount, rowGroupSize };
}

function validateFooter(footer: Uint8Array): { schema: { fields: SchemaField[]; schemaId: number }; rowGroupOffsets: number[]; rowCount: number } {
  const { schema, bytesRead } = deserializeSchema(footer);
  let pos = bytesRead;
  const view = new DataView(footer.buffer, footer.byteOffset, footer.byteLength);
  const rowGroupCount = view.getUint32(pos, true);
  pos += 4;
  const offsets: number[] = [];
  for (let i = 0; i < rowGroupCount; i += 1) {
    offsets.push(Number(view.getBigUint64(pos, true)));
    pos += 8;
  }
  const hasStats = footer[pos];
  pos += 1;
  const statisticsLength = view.getUint32(pos, true);
  pos += 4 + statisticsLength;
  const createdAt = view.getUint32(pos, true);
  pos += 4;
  const modifiedAt = view.getUint32(pos, true);
  pos += 4;
  const numRows = view.getUint32(pos, true);
  pos += 4;
  const checksum = view.getUint32(pos, true);

  const footerWithoutChecksum = footer.subarray(0, pos - 4);
  if (crc32(footerWithoutChecksum) !== checksum) {
    throw new Error('Footer checksum mismatch');
  }

  return { schema, rowGroupOffsets: offsets, rowCount: numRows };
}

function decodeRowGroup(data: Uint8Array, schema: Schema): Array<Array<number | string | Uint8Array | null>> {
  const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
  let pos = 0;
  const rowCount = view.getUint32(pos, true);
  pos += 4;
  const totalSize = view.getUint32(pos, true);
  pos += 4;
  const compressedSize = view.getUint32(pos, true);
  pos += 4;
  const columnCount = view.getUint16(pos, true);
  pos += 2;
  pos += 2;

  if (columnCount !== schema.fieldCount()) {
    throw new Error('Column count mismatch');
  }

  const columns: Array<Array<number | string | Uint8Array | null>> = [];
  for (const field of schema.fields) {
    const encodingId = data[pos];
    const compressionId = data[pos + 1];
    pos += 2;
    if (encodingId !== 0 || compressionId !== 0) {
      throw new Error('Unsupported encoding or compression');
    }
    const uncompressedLen = view.getUint32(pos, true);
    pos += 4;
    const compressedLen = view.getUint32(pos, true);
    pos += 4;
    const nullCount = view.getUint32(pos, true);
    pos += 4;
    pos += 4;
    const columnData = data.subarray(pos, pos + compressedLen);
    const expectedCrc = view.getUint32(pos + compressedLen, true);
    if (crc32(columnData) !== expectedCrc) {
      throw new Error('Column CRC mismatch');
    }

    const columnValues: Array<number | string | Uint8Array | null> = [];
    let columnPos = 0;
    const nullBitmap = field.required ? [] : unpackNullBitmap(columnData.subarray(0, Math.ceil(rowCount / 8)), rowCount);
    if (!field.required) {
      columnPos += Math.ceil(rowCount / 8);
    }

    for (let rowIndex = 0; rowIndex < rowCount; rowIndex += 1) {
      if (!field.required && !nullBitmap[rowIndex]) {
        columnValues.push(null);
        continue;
      }
      const parsed = deserializeValue(field.type, columnData, columnPos);
      columnValues.push(parsed.value);
      columnPos += parsed.bytesRead;
    }

    columns.push(columnValues);
    pos += compressedLen + 4;
  }

  const rows: Array<Array<number | string | Uint8Array | null>> = [];
  for (let i = 0; i < rowCount; i += 1) {
    rows.push(schema.fields.map((_, fieldIndex) => columns[fieldIndex][i]));
  }
  return rows;
}

function encodePayload(payload: unknown): Uint8Array {
  if (typeof payload !== 'object' || payload === null) {
    throw new Error('Invalid payload');
  }
  const { schema, rows } = payload as {
    schema: { fields: SchemaField[]; schemaId: number };
    rows: StoredCell[][];
  };
  const qrdSchema = new Schema(schema.fields);
  const rowValues = rows.map((row) => row.map(deserializeCellFromStorage));

  const header = concatUint8Arrays([
    toUtf8Bytes('QRD\x01'),
    writeUint16(1),
    writeUint16(0),
    writeUint32(qrdSchema.schemaId),
    writeUint32(Math.floor(Date.now() / 1000)),
    writeUint32(rowValues.length),
    writeUint32(qrdSchema.fieldCount()),
    writeUint32(rowValues.length),
    writeUint32(0),
  ]);

  const rowGroup = serializeRowGroup(qrdSchema, rowValues);
  const footer = serializeFooter(qrdSchema, header.length, rowValues.length);
  const footerLength = writeUint32(footer.length);

  return concatUint8Arrays([header, rowGroup, footer, footerLength]);
}

function decodePayload(data: Uint8Array): {
  schema: { fields: SchemaField[]; schemaId: number };
  rows: StoredCell[][];
} {
  if (data.length < 36) {
    throw new Error('Data too short to be a valid QRD file');
  }

  const footerLength = new DataView(data.buffer, data.byteOffset + data.byteLength - 4, 4).getUint32(0, true);
  const footerStart = data.length - 4 - footerLength;
  const footer = data.subarray(footerStart, footerStart + footerLength);
  const parsedFooter = validateFooter(footer);

  const header = parseHeader(data.subarray(0, 32));
  if (header.schemaId !== parsedFooter.schema.schemaId) {
    throw new Error('Schema ID mismatch between header and footer');
  }

  const rowGroupStart = parsedFooter.rowGroupOffsets[0];
  const rowGroupEnd = footerStart;
  const rowGroupData = data.subarray(rowGroupStart, rowGroupEnd);
  const rows = decodeRowGroup(rowGroupData, new Schema(parsedFooter.schema.fields));

  return {
    schema: parsedFooter.schema,
    rows: rows.map((row) => row.map((value) => {
      if (value instanceof Uint8Array) {
        return serializeCellForStorage(value);
      }
      if (typeof value === 'number') {
        return { kind: 'number', value };
      }
      return { kind: 'string', value: String(value) };
    })),
  };
}
