package com.zenipara.qrd;

import java.lang.ref.Cleaner;
import java.util.List;

/**
 * QRD schema definition
 */
public class Schema implements AutoCloseable {
    private static final Cleaner CLEANER = Cleaner.create();

    private final long handle;
    private final long id;
    private final List<Field> fields;
    private final Cleaner.Cleanable cleanable;

    Schema(long handle, long id, List<Field> fields) {
        this.handle = handle;
        this.id = id;
        this.fields = fields;
        this.cleanable = CLEANER.register(this, new Resource(handle));
    }

    private static final class Resource implements Runnable {
        private long handle;

        Resource(long handle) {
            this.handle = handle;
        }

        synchronized void close() {
            if (handle != 0) {
                QRD.INSTANCE.schemaFree(handle);
                handle = 0;
            }
        }

        @Override
        public synchronized void run() {
            close();
        }
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