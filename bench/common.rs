use criterion::{black_box, Criterion};
use qrd_core::{
    error::Result,
    schema::{FieldType, Nullability, Schema, SchemaBuilder},
    writer::{BufferPoolConfig, StreamingWriter, StreamingWriterConfig},
};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    io::{self, Write},
    rc::Rc,
    time::Duration,
};

pub const ROW_COUNTS: [usize; 4] = [100, 1_000, 10_000, 100_000];
pub const STREAMING_BATCH_SIZES: [usize; 4] = [1, 8, 64, 512];
pub const DEFAULT_ROW_GROUP_SIZE: u32 = 1_024;
pub const DEFAULT_COMPRESSION_LEVEL: u8 = 3;
const FIXED_SEED: u64 = 0x5152_442D_4245_4E43;

#[derive(Clone)]
pub struct SharedVecWriter {
    buffer: Rc<RefCell<Vec<u8>>>,
}

impl SharedVecWriter {
    pub fn new() -> Self {
        Self {
            buffer: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.borrow().len()
    }
}

impl Default for SharedVecWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Write for SharedVecWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct BenchmarkDataset {
    pub name: &'static str,
    pub schema: Schema,
    pub rows: Vec<Vec<Vec<u8>>>,
    pub json_rows: Vec<String>,
    pub logical_bytes: usize,
    pub json_bytes: usize,
}

impl BenchmarkDataset {
    fn new(
        name: &'static str,
        schema: Schema,
        rows: Vec<Vec<Vec<u8>>>,
        json_rows: Vec<String>,
    ) -> Self {
        let logical_bytes = rows
            .iter()
            .map(|row| row.iter().map(Vec::len).sum::<usize>())
            .sum();
        let json_bytes = json_rows.iter().map(String::len).sum();

        Self {
            name,
            schema,
            rows,
            json_rows,
            logical_bytes,
            json_bytes,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryRecord {
    pub device_id: u64,
    pub timestamp_micros: u64,
    pub temperature_c: f64,
    pub humidity_pct: f64,
    pub status: String,
    pub region: String,
    pub payload: String,
}

impl TelemetryRecord {
    pub fn to_row(&self) -> Vec<Vec<u8>> {
        vec![
            self.device_id.to_le_bytes().to_vec(),
            self.timestamp_micros.to_le_bytes().to_vec(),
            self.temperature_c.to_le_bytes().to_vec(),
            self.humidity_pct.to_le_bytes().to_vec(),
            self.status.as_bytes().to_vec(),
            self.region.as_bytes().to_vec(),
            self.payload.as_bytes().to_vec(),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiEventRecord {
    pub request_id: u64,
    pub timestamp_micros: u64,
    pub model: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub latency_ms: f64,
    pub success: bool,
    pub payload: String,
}

impl AiEventRecord {
    pub fn to_row(&self) -> Vec<Vec<u8>> {
        vec![
            self.request_id.to_le_bytes().to_vec(),
            self.timestamp_micros.to_le_bytes().to_vec(),
            self.model.as_bytes().to_vec(),
            self.prompt_tokens.to_le_bytes().to_vec(),
            self.completion_tokens.to_le_bytes().to_vec(),
            self.latency_ms.to_le_bytes().to_vec(),
            vec![u8::from(self.success)],
            self.payload.as_bytes().to_vec(),
        ]
    }
}

#[derive(Debug, Clone)]
pub struct EntropyRecord {
    pub sequence: u64,
    pub checksum: u64,
    pub blob: Vec<u8>,
    pub label: String,
}

impl EntropyRecord {
    pub fn to_row(&self) -> Vec<Vec<u8>> {
        vec![
            self.sequence.to_le_bytes().to_vec(),
            self.checksum.to_le_bytes().to_vec(),
            self.blob.clone(),
            self.label.as_bytes().to_vec(),
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepetitiveRecord {
    pub sequence: u64,
    pub category: String,
    pub status: String,
    pub payload: String,
}

impl RepetitiveRecord {
    pub fn to_row(&self) -> Vec<Vec<u8>> {
        vec![
            self.sequence.to_le_bytes().to_vec(),
            self.category.as_bytes().to_vec(),
            self.status.as_bytes().to_vec(),
            self.payload.as_bytes().to_vec(),
        ]
    }
}

pub fn criterion_config() -> Criterion {
    Criterion::default()
        .sample_size(20)
        .warm_up_time(Duration::from_secs(3))
        .measurement_time(Duration::from_secs(10))
        .noise_threshold(0.02)
}

pub fn schema(fields: &[(&str, FieldType)]) -> Schema {
    let mut builder = SchemaBuilder::new();

    for (name, field_type) in fields.iter().copied() {
        builder = builder
            .add_field(name, field_type, Nullability::Required)
            .expect("benchmark schema is valid");
    }

    builder.build().expect("benchmark schema is valid")
}

pub fn telemetry_schema() -> Schema {
    schema(&[
        ("device_id", FieldType::UInt64),
        ("timestamp_micros", FieldType::Timestamp),
        ("temperature_c", FieldType::Float64),
        ("humidity_pct", FieldType::Float64),
        ("status", FieldType::String),
        ("region", FieldType::String),
        ("payload", FieldType::Blob),
    ])
}

pub fn ai_event_schema() -> Schema {
    schema(&[
        ("request_id", FieldType::UInt64),
        ("timestamp_micros", FieldType::Timestamp),
        ("model", FieldType::String),
        ("prompt_tokens", FieldType::UInt32),
        ("completion_tokens", FieldType::UInt32),
        ("latency_ms", FieldType::Float64),
        ("success", FieldType::Boolean),
        ("payload", FieldType::Blob),
    ])
}

pub fn entropy_schema() -> Schema {
    schema(&[
        ("sequence", FieldType::UInt64),
        ("checksum", FieldType::UInt64),
        ("blob", FieldType::Blob),
        ("label", FieldType::String),
    ])
}

pub fn repetitive_schema() -> Schema {
    schema(&[
        ("sequence", FieldType::UInt64),
        ("category", FieldType::String),
        ("status", FieldType::String),
        ("payload", FieldType::Blob),
    ])
}

pub fn telemetry_dataset(row_count: usize) -> BenchmarkDataset {
    let schema = telemetry_schema();
    let status_cycle = ["ok", "warn", "degraded"];
    let region_cycle = ["us-east-1", "us-west-2", "eu-central-1", "ap-southeast-1"];

    let mut rows = Vec::with_capacity(row_count);
    let mut json_rows = Vec::with_capacity(row_count);

    for index in 0..row_count {
        let record = TelemetryRecord {
            device_id: 10_000 + index as u64,
            timestamp_micros: 1_725_000_000_000_000 + (index as u64 * 1_000),
            temperature_c: 18.5 + ((index % 37) as f64 * 0.11),
            humidity_pct: 40.0 + ((index % 21) as f64 * 0.19),
            status: status_cycle[index % status_cycle.len()].to_string(),
            region: region_cycle[index % region_cycle.len()].to_string(),
            payload: format!(
                "sensor:{}:{}:{}:{}",
                index % 12,
                index % 31,
                index % 7,
                index % 5
            ),
        };

        json_rows.push(serde_json::to_string(&record).expect("telemetry JSON must serialize"));
        rows.push(record.to_row());
    }

    BenchmarkDataset::new("telemetry", schema, rows, json_rows)
}

pub fn ai_event_dataset(row_count: usize) -> BenchmarkDataset {
    let schema = ai_event_schema();
    let models = ["qrd-mini", "qrd-pro", "qrd-reasoner"];
    let payload_templates = [
        "chat_completion:short_context",
        "chat_completion:tool_call",
        "chat_completion:retrieval_augmented",
        "chat_completion:long_context",
    ];

    let mut rows = Vec::with_capacity(row_count);
    let mut json_rows = Vec::with_capacity(row_count);

    for index in 0..row_count {
        let record = AiEventRecord {
            request_id: 50_000 + index as u64,
            timestamp_micros: 1_725_100_000_000_000 + (index as u64 * 750),
            model: models[index % models.len()].to_string(),
            prompt_tokens: 96 + ((index % 8) as u32 * 32),
            completion_tokens: 32 + ((index % 5) as u32 * 24),
            latency_ms: 24.0 + ((index % 29) as f64 * 1.7),
            success: index % 13 != 0,
            payload: format!(
                "{}::trace={}::tenant={}",
                payload_templates[index % payload_templates.len()],
                index % 19,
                index % 11
            ),
        };

        json_rows.push(serde_json::to_string(&record).expect("AI event JSON must serialize"));
        rows.push(record.to_row());
    }

    BenchmarkDataset::new("ai_events", schema, rows, json_rows)
}

pub fn entropy_dataset(row_count: usize) -> BenchmarkDataset {
    let schema = entropy_schema();
    let mut rng = StdRng::seed_from_u64(FIXED_SEED ^ row_count as u64);

    let mut rows = Vec::with_capacity(row_count);

    for index in 0..row_count {
        let mut blob = vec![0u8; 96 + (index % 11) * 16];
        rng.fill_bytes(&mut blob);

        let checksum = blob.iter().fold(0u64, |acc, byte| {
            acc.wrapping_mul(131).wrapping_add(*byte as u64)
        });

        rows.push(
            EntropyRecord {
                sequence: index as u64,
                checksum,
                blob,
                label: format!("entropy-{}", index % 17),
            }
            .to_row(),
        );
    }

    BenchmarkDataset::new("entropy", schema, rows, Vec::new())
}

pub fn repetitive_dataset(row_count: usize) -> BenchmarkDataset {
    let schema = repetitive_schema();
    let mut rows = Vec::with_capacity(row_count);
    let mut json_rows = Vec::with_capacity(row_count);

    for index in 0..row_count {
        let record = RepetitiveRecord {
            sequence: index as u64,
            category: "heartbeat".to_string(),
            status: "ok".to_string(),
            payload: "static-payload-for-compression".repeat(2),
        };

        json_rows.push(serde_json::to_string(&record).expect("repetitive JSON must serialize"));
        rows.push(record.to_row());
    }

    BenchmarkDataset::new("repetitive", schema, rows, json_rows)
}

pub fn write_rows(
    schema: &Schema,
    rows: &[Vec<Vec<u8>>],
    row_group_size: u32,
    compression_level: u8,
) -> Result<usize> {
    let sink = SharedVecWriter::new();
    let capture = sink.clone();
    let mut config = StreamingWriterConfig::default();
    config.row_group_size = row_group_size;
    config.compression_level = compression_level;
    config.buffer_pool_config = BufferPoolConfig::default();

    let mut writer = StreamingWriter::with_config(sink, schema.clone(), config)?;

    for row in rows {
        writer.write_row(black_box(row.clone()))?;
    }

    writer.finish()?;
    Ok(capture.len())
}

pub fn report_metrics(
    benchmark: &str,
    dataset: &str,
    rows: usize,
    logical_bytes: usize,
    json_bytes: usize,
    qrd_bytes: usize,
    elapsed: Duration,
) {
    let elapsed_secs = elapsed.as_secs_f64();
    let rows_per_sec = rows as f64 / elapsed_secs;
    let logical_mb_per_sec = logical_bytes as f64 / (1024.0 * 1024.0) / elapsed_secs;
    let compression_ratio = logical_bytes as f64 / qrd_bytes as f64;
    let bytes_per_row = qrd_bytes as f64 / rows as f64;
    let overhead_per_row = qrd_bytes.saturating_sub(logical_bytes) as f64 / rows as f64;

    println!(
        "[{benchmark}] dataset={dataset} rows={rows} rows_per_sec={rows_per_sec:.2} logical_mb_per_sec={logical_mb_per_sec:.2} compression_ratio={compression_ratio:.3} json_bytes={json_bytes} qrd_bytes={qrd_bytes} bytes_per_row={bytes_per_row:.2} overhead_per_row={overhead_per_row:.2}",
    );
}
