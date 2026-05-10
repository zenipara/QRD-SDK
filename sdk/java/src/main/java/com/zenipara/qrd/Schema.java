package com.zenipara.qrd;

import java.util.List;

/**
 * QRD schema definition
 */
public class Schema implements AutoCloseable {
    private final long handle;
    private final long id;
    private final List<Field> fields;

    Schema(long handle, long id, List<Field> fields) {
        this.handle = handle;
        this.id = id;
        this.fields = fields;
    }

    /**
     * Get the schema ID
     */
    public long getId() {
        return id;
    }

    /**
     * Get the fields in this schema
     */
    public List<Field> getFields() {
        return fields;
    }

    /**
     * Get field count
     */
    public int getFieldCount() {
        return fields.size();
    }

    long getHandle() {
        return handle;
    }

    @Override
    public void close() {
        QRD.INSTANCE.qrd_schema_free(handle);
    }

    @Override
    public String toString() {
        return String.format("Schema{id=%d, fields=%d}", id, fields.size());
    }
}