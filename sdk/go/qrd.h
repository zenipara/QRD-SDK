#ifndef QRD_H
#define QRD_H

#include <stdint.h>
#include <stddef.h>

// Opaque types
typedef struct QrdSchema QrdSchema;
typedef struct QrdWriter QrdWriter;
typedef struct QrdReader QrdReader;
typedef struct QrdRow QrdRow;

// Schema API
QrdSchema* qrd_schema_new(void);
void qrd_schema_free(QrdSchema* schema);
int qrd_schema_add_field(QrdSchema* schema, const char* name, int field_type, int nullability, const char* metadata);
uint64_t qrd_schema_id(const QrdSchema* schema);
size_t qrd_schema_field_count(const QrdSchema* schema);

// Writer API
QrdWriter* qrd_writer_new(QrdSchema* schema);
void qrd_writer_free(QrdWriter* writer);
int qrd_writer_write_row(QrdWriter* writer, QrdRow* row);
int qrd_writer_finish(QrdWriter* writer, uint8_t** data, size_t* size);

// Reader API
QrdReader* qrd_reader_new(const uint8_t* data, size_t size);
void qrd_reader_free(QrdReader* reader);
QrdSchema* qrd_reader_schema(QrdReader* reader);
uint64_t qrd_reader_row_count(QrdReader* reader);
QrdRow* qrd_reader_read_row(QrdReader* reader);

// Row API
QrdRow* qrd_row_new(void);
void qrd_row_free(QrdRow* row);
size_t qrd_row_field_count(const QrdRow* row);
int qrd_row_add_bytes(QrdRow* row, const uint8_t* data, size_t size);
int qrd_row_add_int64(QrdRow* row, int64_t value);
int qrd_row_add_float64(QrdRow* row, double value);
int qrd_row_add_string(QrdRow* row, const char* value);

#endif // QRD_H