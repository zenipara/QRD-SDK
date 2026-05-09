#ifndef QRD_H
#define QRD_H

#include <stdint.h>
#include <stddef.h>

// Opaque types
typedef struct QrdSchemaBuilder QrdSchemaBuilder;
typedef struct QrdSchema QrdSchema;
typedef struct QrdWriter QrdWriter;
typedef struct QrdReader QrdReader;

// Error handling
const char* qrd_last_error(void);

// Schema Builder API
QrdSchemaBuilder* qrd_schema_builder_new(void);
void qrd_schema_builder_free(QrdSchemaBuilder* builder);
int qrd_schema_builder_add_field(QrdSchemaBuilder* builder, const char* name, int field_type, int nullability);
QrdSchema* qrd_schema_builder_build(QrdSchemaBuilder* builder);

// Schema API
void qrd_schema_free(QrdSchema* schema);
uint32_t qrd_schema_field_count(const QrdSchema* schema);

// Writer API
QrdWriter* qrd_writer_new(const char* path, QrdSchema* schema);
void qrd_writer_free(QrdWriter* writer);
int qrd_writer_write_row(QrdWriter* writer, uint32_t column_count, const uint8_t** data_ptrs, const uint32_t* data_lens);
int qrd_writer_finish(QrdWriter* writer);

// Reader API
QrdReader* qrd_reader_new(const char* path);
void qrd_reader_free(QrdReader* reader);
uint32_t qrd_reader_row_count(const QrdReader* reader);

#endif // QRD_H