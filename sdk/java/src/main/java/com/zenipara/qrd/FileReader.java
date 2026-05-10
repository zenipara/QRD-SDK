package com.zenipara.qrd;

import com.sun.jna.Pointer;
import com.sun.jna.ptr.IntByReference;
import java.lang.ref.Cleaner;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;

/**
 * Reader for QRD files
 */
public class FileReader implements AutoCloseable {
    private static final Cleaner CLEANER = Cleaner.create();

    private final long handle;
    private final Schema schema;
    private final Cleaner.Cleanable cleanable;

    FileReader(long handle, Schema schema) {
        this.handle = handle;
        this.schema = schema;
        this.cleanable = CLEANER.register(this, new Resource(handle));
    }

    private static final class Resource implements Runnable {
        private long handle;

        Resource(long handle) {
            this.handle = handle;
        }

        synchronized void close() {
            if (handle != 0) {
                QRD.INSTANCE.qrd_reader_free(handle);
                handle = 0;
            }
        }

        @Override
        public synchronized void run() {
            close();
        }
    }

    /**
     * Get the schema
     */
    public Schema getSchema() {
        return schema;
    }

    /**
     * Get the number of rows
     */
    public long getRowCount() {
        return QRD.INSTANCE.qrd_reader_row_count(handle);
    }

    /**
     * Read the next row
     */
    public List<Object> readRow() throws QRDException {
        long rowHandle = QRD.INSTANCE.qrd_reader_read_row(handle);
        if (rowHandle == 0) {
            return null; // EOF
        }

        try {
            List<Object> row = new ArrayList<>();
            int fieldCount = QRD.INSTANCE.qrd_row_field_count(rowHandle);
            int schemaFieldCount = schema.getFieldCount();

            for (int i = 0; i < fieldCount; i++) {
                if (i >= schemaFieldCount) {
                    row.add(null);
                    continue;
                }

                Field field = schema.getFields().get(i);
                IntByReference sizeRef = new IntByReference();
                Pointer fieldPtr = QRD.INSTANCE.qrd_row_get_bytes(rowHandle, i, sizeRef);
                if (fieldPtr == null) {
                    row.add(null);
                    continue;
                }

                byte[] valueBytes = fieldPtr.getByteArray(0, sizeRef.getValue());
                switch (field.getFieldType()) {
                    case INT64:
                        if (valueBytes.length == Long.BYTES) {
                            row.add(ByteBuffer.wrap(valueBytes).order(ByteOrder.LITTLE_ENDIAN).getLong());
                        } else {
                            row.add(null);
                        }
                        break;
                    case FLOAT64:
                        if (valueBytes.length == Double.BYTES) {
                            row.add(ByteBuffer.wrap(valueBytes).order(ByteOrder.LITTLE_ENDIAN).getDouble());
                        } else {
                            row.add(null);
                        }
                        break;
                    case STRING:
                        if (valueBytes.length >= Integer.BYTES) {
                            int length = ByteBuffer.wrap(valueBytes, 0, Integer.BYTES)
                                .order(ByteOrder.LITTLE_ENDIAN)
                                .getInt();
                            if (length == 0) {
                                row.add("");
                            } else if (valueBytes.length >= Integer.BYTES + length) {
                                row.add(new String(valueBytes, Integer.BYTES, length, StandardCharsets.UTF_8));
                            } else {
                                row.add(null);
                            }
                        } else {
                            row.add(null);
                        }
                        break;
                    default:
                        row.add(null);
                        break;
                }
            }

            return row;
        } finally {
            QRD.INSTANCE.qrd_row_free(rowHandle);
        }
    }

    /**
     * Read all rows
     */
    public List<List<Object>> readAllRows() throws QRDException {
        List<List<Object>> rows = new ArrayList<>();
        List<Object> row;

        while ((row = readRow()) != null) {
            rows.add(row);
        }

        return rows;
    }

    @Override
    public void close() {
        QRD.INSTANCE.qrd_reader_free(handle);
    }
}