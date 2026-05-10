#include "qrd.h"
#include <stdlib.h>
#include <string.h>

typedef struct FFISchema QrdSchemaFFI;
typedef struct FFIWriter QrdWriterFFI;
typedef struct FFIReader QrdReaderFFI;
typedef struct FFIRow QrdRowFFI;

extern QrdSchemaFFI* qrd_schema_new_ffi(void);
extern void qrd_schema_free_ffi(QrdSchemaFFI* schema);
extern int qrd_schema_add_field_ffi(QrdSchemaFFI* schema, const char* name, int field_type, int nullability, const char* metadata);
extern uint64_t qrd_schema_id_ffi(const QrdSchemaFFI* schema);
extern size_t qrd_schema_field_count_ffi(const QrdSchemaFFI* schema);

extern QrdWriterFFI* qrd_writer_new_ffi(const QrdSchemaFFI* schema);
extern void qrd_writer_free_ffi(QrdWriterFFI* writer);
extern int qrd_writer_write_row_ffi(QrdWriterFFI* writer, const QrdRowFFI* row);
extern int qrd_writer_finish_ffi(QrdWriterFFI* writer, uint8_t** data, size_t* size);

extern QrdReaderFFI* qrd_reader_new_ffi(const uint8_t* data, size_t size);
extern void qrd_reader_free_ffi(QrdReaderFFI* reader);
extern QrdSchemaFFI* qrd_reader_schema_ffi(QrdReaderFFI* reader);
extern uint64_t qrd_reader_row_count_ffi(QrdReaderFFI* reader);
extern QrdRowFFI* qrd_reader_read_row_ffi(QrdReaderFFI* reader);

extern QrdRowFFI* qrd_row_new_ffi(void);
extern void qrd_row_free_ffi(QrdRowFFI* row);
extern size_t qrd_row_field_count_ffi(const QrdRowFFI* row);
extern int qrd_row_add_bytes_ffi(QrdRowFFI* row, const uint8_t* data, size_t size);
extern int qrd_row_add_int64_ffi(QrdRowFFI* row, int64_t value);
extern int qrd_row_add_float64_ffi(QrdRowFFI* row, double value);
extern int qrd_row_add_string_ffi(QrdRowFFI* row, const char* value);

// Schema API
QrdSchema* qrd_schema_new(void) {
    return (QrdSchema*)qrd_schema_new_ffi();
}

void qrd_schema_free(QrdSchema* schema) {
    qrd_schema_free_ffi((QrdSchemaFFI*)schema);
}

int qrd_schema_add_field(QrdSchema* schema, const char* name, int field_type, int nullability, const char* metadata) {
    return qrd_schema_add_field_ffi((QrdSchemaFFI*)schema, name, field_type, nullability, metadata);
}

uint64_t qrd_schema_id(const QrdSchema* schema) {
    return qrd_schema_id_ffi((const QrdSchemaFFI*)schema);
}

size_t qrd_schema_field_count(const QrdSchema* schema) {
    return qrd_schema_field_count_ffi((const QrdSchemaFFI*)schema);
}

// Writer API
QrdWriter* qrd_writer_new(QrdSchema* schema) {
    return (QrdWriter*)qrd_writer_new_ffi((QrdSchemaFFI*)schema);
}

void qrd_writer_free(QrdWriter* writer) {
    qrd_writer_free_ffi((QrdWriterFFI*)writer);
}

int qrd_writer_write_row(QrdWriter* writer, QrdRow* row) {
    return qrd_writer_write_row_ffi((QrdWriterFFI*)writer, (QrdRowFFI*)row);
}

int qrd_writer_finish(QrdWriter* writer, uint8_t** data, size_t* size) {
    return qrd_writer_finish_ffi((QrdWriterFFI*)writer, data, size);
}

// Reader API
QrdReader* qrd_reader_new(const uint8_t* data, size_t size) {
    return (QrdReader*)qrd_reader_new_ffi(data, size);
}

void qrd_reader_free(QrdReader* reader) {
    qrd_reader_free_ffi((QrdReaderFFI*)reader);
}

QrdSchema* qrd_reader_schema(QrdReader* reader) {
    return (QrdSchema*)qrd_reader_schema_ffi((QrdReaderFFI*)reader);
}

uint64_t qrd_reader_row_count(QrdReader* reader) {
    return qrd_reader_row_count_ffi((QrdReaderFFI*)reader);
}

QrdRow* qrd_reader_read_row(QrdReader* reader) {
    return (QrdRow*)qrd_reader_read_row_ffi((QrdReaderFFI*)reader);
}

// Row API
QrdRow* qrd_row_new(void) {
    return (QrdRow*)qrd_row_new_ffi();
}

void qrd_row_free(QrdRow* row) {
    qrd_row_free_ffi((QrdRowFFI*)row);
}

size_t qrd_row_field_count(const QrdRow* row) {
    return qrd_row_field_count_ffi((const QrdRowFFI*)row);
}

int qrd_row_add_bytes(QrdRow* row, const uint8_t* data, size_t size) {
    return qrd_row_add_bytes_ffi((QrdRowFFI*)row, data, size);
}

int qrd_row_add_int64(QrdRow* row, int64_t value) {
    return qrd_row_add_int64_ffi((QrdRowFFI*)row, value);
}

int qrd_row_add_float64(QrdRow* row, double value) {
    return qrd_row_add_float64_ffi((QrdRowFFI*)row, value);
}

int qrd_row_add_string(QrdRow* row, const char* value) {
    return qrd_row_add_string_ffi((QrdRowFFI*)row, value);
}