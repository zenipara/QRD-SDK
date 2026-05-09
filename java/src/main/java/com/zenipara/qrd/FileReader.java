package com.zenipara.qrd;

import java.util.ArrayList;
import java.util.List;

/**
 * Reader for QRD files
 */
public class FileReader implements AutoCloseable {
    private final long handle;
    private final Schema schema;

    FileReader(long handle, Schema schema) {
        this.handle = handle;
        this.schema = schema;
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
        return QRD.INSTANCE.readerRowCount(handle);
    }

    /**
     * Read the next row
     */
    public List<Object> readRow() throws QRDException {
        long rowHandle = QRD.INSTANCE.readerReadRow(handle);
        if (rowHandle == 0) {
            return null; // EOF
        }

        try {
            List<Object> row = new ArrayList<>();
            int fieldCount = QRD.INSTANCE.rowFieldCount(rowHandle);

            for (int i = 0; i < fieldCount; i++) {
                // This is simplified - in practice you'd need to read actual field values
                // based on the schema field types
                row.add(null); // Placeholder
            }

            return row;
        } finally {
            QRD.INSTANCE.rowFree(rowHandle);
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
        QRD.INSTANCE.readerFree(handle);
    }
}