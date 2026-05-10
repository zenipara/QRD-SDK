#include "qrd.h"
#include <stdlib.h>
#include <string.h>

typedef struct FFISchemaBuilder FFISchemaBuilder;
typedef struct FFISchema FFISchema;
typedef struct FFIWriter FFIWriter;
typedef struct FFIReader FFIReader;
typedef struct FFIRow FFIRow;

typedef struct QrdSchema {
    FFISchemaBuilder* builder;
    FFISchema* built_schema;
    int owns_built_schema;
} QrdSchema;

typedef struct QrdWriter {
    FFIWriter* ffi_writer;
} QrdWriter;

typedef struct QrdReader {
    FFIReader* ffi_reader;
} QrdReader;

extern const char* qrd_last_error_ffi(void);
extern FFISchemaBuilder* qrd_schema_builder_new_ffi(void);
extern void qrd_schema_builder_free_ffi(FFISchemaBuilder* builder);
extern int qrd_schema_builder_add_field_ffi(FFISchemaBuilder* builder, const char* name, int field_type, int nullability);
extern FFISchema* qrd_schema_builder_build_ffi(FFISchemaBuilder* builder);
extern void qrd_schema_free_ffi(FFISchema* schema);
extern uint64_t qrd_schema_id_ffi(const FFISchema* schema);
extern size_t qrd_schema_field_count_ffi(const FFISchema* schema);
extern FFIWriter* qrd_writer_new_ffi(const FFISchema* schema);
extern void qrd_writer_free_ffi(FFIWriter* writer);
extern int qrd_writer_write_row_ffi(FFIWriter* writer, const FFIRow* row);
extern int qrd_writer_finish_ffi(FFIWriter* writer, uint8_t** data, size_t* size);
extern FFIReader* qrd_reader_new_ffi(const uint8_t* data, size_t size);
extern void qrd_reader_free_ffi(FFIReader* reader);
extern FFISchema* qrd_reader_schema_ffi(FFIReader* reader);
extern uint64_t qrd_reader_row_count_ffi(FFIReader* reader);
extern FFIRow* qrd_reader_read_row_ffi(FFIReader* reader);
extern FFIRow* qrd_row_new_ffi(void);
extern void qrd_row_free_ffi(FFIRow* row);
extern size_t qrd_row_field_count_ffi(const FFIRow* row);
extern int qrd_row_add_bytes_ffi(FFIRow* row, const uint8_t* data, size_t size);
extern int qrd_row_add_int64_ffi(FFIRow* row, int64_t value);
extern int qrd_row_add_float64_ffi(FFIRow* row, double value);
extern int qrd_row_add_string_ffi(FFIRow* row, const char* value);

const char* qrd_last_error(void) {
    return qrd_last_error_ffi();
}

QrdSchemaBuilder* qrd_schema_builder_new(void) {
    return (QrdSchemaBuilder*)qrd_schema_builder_new_ffi();
}

void qrd_schema_builder_free(QrdSchemaBuilder* builder) {
    qrd_schema_builder_free_ffi((FFISchemaBuilder*)builder);
}

int qrd_schema_builder_add_field(QrdSchemaBuilder* builder, const char* name, int field_type, int nullability) {
    return qrd_schema_builder_add_field_ffi((FFISchemaBuilder*)builder, name, field_type, nullability);
}

QrdSchema* qrd_schema_builder_build(QrdSchemaBuilder* builder) {
    QrdSchema* schema = (QrdSchema*)malloc(sizeof(QrdSchema));
    if (schema == NULL) {
        return NULL;
    }

    schema->builder = NULL;
    schema->built_schema = qrd_schema_builder_build_ffi((FFISchemaBuilder*)builder);
    schema->owns_built_schema = 1;

    if (schema->built_schema == NULL) {
        free(schema);
        return NULL;
    }

    return schema;
}

QrdSchema* qrd_schema_new(void) {
    QrdSchema* schema = (QrdSchema*)malloc(sizeof(QrdSchema));
    if (schema == NULL) {
        return NULL;
    }

    schema->builder = qrd_schema_builder_new_ffi();
    schema->built_schema = NULL;
    schema->owns_built_schema = 0;

    if (schema->builder == NULL) {
        free(schema);
        return NULL;
    }

    return schema;
}

void qrd_schema_free(QrdSchema* schema) {
    if (schema == NULL) {
        return;
    }

    if (schema->builder != NULL) {
        qrd_schema_builder_free_ffi(schema->builder);
    }

    if (schema->built_schema != NULL && schema->owns_built_schema) {
        qrd_schema_free_ffi(schema->built_schema);
    }

    free(schema);
}

int qrd_schema_add_field(QrdSchema* schema, const char* name, int field_type, int nullability, const char* metadata) {
    (void)metadata;
    if (schema == NULL || schema->builder == NULL) {
        return -1;
    }
    return qrd_schema_builder_add_field_ffi(schema->builder, name, field_type, nullability);
}

uint64_t qrd_schema_id(const QrdSchema* schema) {
    if (schema == NULL || schema->built_schema == NULL) {
        return 0;
    }
    return qrd_schema_id_ffi(schema->built_schema);
}

size_t qrd_schema_field_count(const QrdSchema* schema) {
    if (schema == NULL || schema->built_schema == NULL) {
        return 0;
    }
    return qrd_schema_field_count_ffi(schema->built_schema);
}

QrdWriter* qrd_writer_new(QrdSchema* schema) {
    if (schema == NULL) {
        return NULL;
    }

    if (schema->built_schema == NULL) {
        if (schema->builder == NULL) {
            return NULL;
        }

        schema->built_schema = qrd_schema_builder_build_ffi(schema->builder);
        schema->builder = NULL;
        schema->owns_built_schema = 1;
    }

    if (schema->built_schema == NULL) {
        return NULL;
    }

    QrdWriter* writer = (QrdWriter*)malloc(sizeof(QrdWriter));
    if (writer == NULL) {
        return NULL;
    }

    writer->ffi_writer = qrd_writer_new_ffi(schema->built_schema);
    if (writer->ffi_writer == NULL) {
        free(writer);
        return NULL;
    }

    return writer;
}

void qrd_writer_free(QrdWriter* writer) {
    if (writer == NULL) {
        return;
    }

    if (writer->ffi_writer != NULL) {
        qrd_writer_free_ffi(writer->ffi_writer);
    }

    free(writer);
}

int qrd_writer_write_row(QrdWriter* writer, QrdRow* row) {
    if (writer == NULL) {
        return -1;
    }
    return qrd_writer_write_row_ffi(writer->ffi_writer, (const FFIRow*)row);
}

int qrd_writer_finish(QrdWriter* writer, uint8_t** data, size_t* size) {
    if (writer == NULL) {
        return -1;
    }
    return qrd_writer_finish_ffi(writer->ffi_writer, data, size);
}

QrdReader* qrd_reader_new(const uint8_t* data, size_t size) {
    QrdReader* reader = (QrdReader*)malloc(sizeof(QrdReader));
    if (reader == NULL) {
        return NULL;
    }

    reader->ffi_reader = qrd_reader_new_ffi(data, size);
    if (reader->ffi_reader == NULL) {
        free(reader);
        return NULL;
    }

    return reader;
}

void qrd_reader_free(QrdReader* reader) {
    if (reader == NULL) {
        return;
    }

    if (reader->ffi_reader != NULL) {
        qrd_reader_free_ffi(reader->ffi_reader);
    }

    free(reader);
}

QrdSchema* qrd_reader_schema(QrdReader* reader) {
    if (reader == NULL) {
        return NULL;
    }

    FFISchema* ffi_schema = qrd_reader_schema_ffi(reader->ffi_reader);
    if (ffi_schema == NULL) {
        return NULL;
    }

    QrdSchema* schema = (QrdSchema*)malloc(sizeof(QrdSchema));
    if (schema == NULL) {
        return NULL;
    }

    schema->builder = NULL;
    schema->built_schema = ffi_schema;
    schema->owns_built_schema = 0;
    return schema;
}

uint64_t qrd_reader_row_count(QrdReader* reader) {
    if (reader == NULL) {
        return 0;
    }
    return qrd_reader_row_count_ffi(reader->ffi_reader);
}

QrdRow* qrd_reader_read_row(QrdReader* reader) {
    if (reader == NULL) {
        return NULL;
    }
    return (QrdRow*)qrd_reader_read_row_ffi(reader->ffi_reader);
}

QrdRow* qrd_row_new(void) {
    return (QrdRow*)qrd_row_new_ffi();
}

void qrd_row_free(QrdRow* row) {
    qrd_row_free_ffi((FFIRow*)row);
}

size_t qrd_row_field_count(const QrdRow* row) {
    return qrd_row_field_count_ffi((const FFIRow*)row);
}

int qrd_row_add_bytes(QrdRow* row, const uint8_t* data, size_t size) {
    return qrd_row_add_bytes_ffi((FFIRow*)row, data, size);
}

int qrd_row_add_int64(QrdRow* row, int64_t value) {
    return qrd_row_add_int64_ffi((FFIRow*)row, value);
}

int qrd_row_add_float64(QrdRow* row, double value) {
    return qrd_row_add_float64_ffi((FFIRow*)row, value);
}

int qrd_row_add_string(QrdRow* row, const char* value) {
    return qrd_row_add_string_ffi((FFIRow*)row, value);
}
