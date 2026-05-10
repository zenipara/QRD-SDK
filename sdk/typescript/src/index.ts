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
  private rows: Array<Array<number | string | Uint8Array>> = [];
  private finished = false;

  constructor(schema: Schema) {
    this.schema = schema;
  }

  writeRow(row: Array<number | string | Uint8Array>): void {
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
  private readonly rows: Array<Array<number | string | Uint8Array>>;
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

  readRow(): Array<number | string | Uint8Array> | null {
    if (this.cursor >= this.rows.length) {
      return null;
    }
    const row = this.rows[this.cursor];
    this.cursor += 1;
    return row.map(cloneCell);
  }

  readAllRows(): Array<Array<number | string | Uint8Array>> {
    const rows: Array<Array<number | string | Uint8Array>> = [];
    let row: Array<number | string | Uint8Array> | null;
    while ((row = this.readRow()) !== null) {
      rows.push(row);
    }
    return rows;
  }
}

export async function createQrdFile(
  fields: Array<{ name: string; type: string }>,
  rows: Array<Array<number | string | Uint8Array>>,
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
    return new Uint8Array(Buffer.from(JSON.stringify(value), 'utf8'));
  }

  static serializeFloat64(value: number): Uint8Array {
    return new Uint8Array(Buffer.from(JSON.stringify(value), 'utf8'));
  }

  static serializeString(value: string): Uint8Array {
    return new Uint8Array(Buffer.from(value, 'utf8'));
  }

  static deserializeInt64(bytes: Uint8Array): number {
    return Number.parseInt(Buffer.from(bytes).toString('utf8'), 10);
  }

  static deserializeFloat64(bytes: Uint8Array): number {
    return Number.parseFloat(Buffer.from(bytes).toString('utf8'));
  }

  static deserializeString(bytes: Uint8Array): string {
    return Buffer.from(bytes).toString('utf8');
  }
}

function parseFieldType(type: string): FieldType {
  const normalized = type.toUpperCase();
  if (normalized in FieldType) {
    return normalized as FieldType;
  }
  throw new Error(`Unknown field type: ${type}`);
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
  | { kind: 'bytes'; value: number[] };

function cloneCell(value: number | string | Uint8Array): number | string | Uint8Array {
  if (value instanceof Uint8Array) {
    return new Uint8Array(value);
  }
  return value;
}

function serializeCellForStorage(value: number | string | Uint8Array): StoredCell {
  if (value instanceof Uint8Array) {
    return { kind: 'bytes', value: Array.from(value) };
  }
  if (typeof value === 'number') {
    return { kind: 'number', value };
  }
  return { kind: 'string', value };
}

function deserializeCellFromStorage(value: StoredCell): number | string | Uint8Array {
  if (value.kind === 'bytes') {
    return new Uint8Array(value.value);
  }
  return value.value;
}

function encodePayload(payload: unknown): Uint8Array {
  return new Uint8Array(Buffer.from(JSON.stringify(payload), 'utf8'));
}

function decodePayload(data: Uint8Array): {
  schema: { fields: SchemaField[]; schemaId: number };
  rows: StoredCell[][];
} {
  const text = Buffer.from(data).toString('utf8');
  return JSON.parse(text) as {
    schema: { fields: SchemaField[]; schemaId: number };
    rows: StoredCell[][];
  };
}
