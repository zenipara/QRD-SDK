package com.zenipara.qrd;

import java.util.List;

/**
 * Writer for QRD files
 */
public class FileWriter implements AutoCloseable {
    private final long handle;
    private boolean finished = false;

    FileWriter(long handle) {
        this.handle = handle;
    }

    /**
     * Write a row of data
     */
    public void writeRow(Object... values) throws QRDException {
        if (finished) {
            throw new QRDException("Writer is already finished");
        }

        long rowHandle = QRD.INSTANCE.rowNew();
        if (rowHandle == 0) {
            throw new QRDException("Failed to create row");
        }

        try {
            for (Object value : values) {
                addValueToRow(rowHandle, value);
            }

            int result = QRD.INSTANCE.writerWriteRow(handle, rowHandle);
            if (result != 0) {
                throw new QRDException("Failed to write row");
            }
        } finally {
            QRD.INSTANCE.rowFree(rowHandle);
        }
    }

    /**
     * Write a row from a list of values
     */
    public void writeRow(List<Object> values) throws QRDException {
        writeRow(values.toArray());
    }

    /**
     * Finish writing and get the data
     */
    public byte[] finish() throws QRDException {
        if (finished) {
            throw new QRDException("Writer is already finished");
        }

        finished = true;

        // This is a simplified implementation
        // In practice, you'd need to implement proper buffer management
        // For now, return empty array as placeholder
        return new byte[0];
    }

    private void addValueToRow(long rowHandle, Object value) throws QRDException {
        if (value == null) {
            // Handle null values - would need proper implementation
            return;
        }

        if (value instanceof Long) {
            QRD.INSTANCE.rowAddInt64(rowHandle, (Long) value);
        } else if (value instanceof Double) {
            QRD.INSTANCE.rowAddFloat64(rowHandle, (Double) value);
        } else if (value instanceof String) {
            QRD.INSTANCE.rowAddString(rowHandle, (String) value);
        } else if (value instanceof Integer) {
            QRD.INSTANCE.rowAddInt64(rowHandle, ((Integer) value).longValue());
        } else if (value instanceof Float) {
            QRD.INSTANCE.rowAddFloat64(rowHandle, ((Float) value).doubleValue());
        } else if (value instanceof Boolean) {
            // Would need to implement boolean support
            throw new QRDException("Boolean values not yet supported");
        } else {
            throw new QRDException("Unsupported value type: " + value.getClass().getName());
        }
    }

    @Override
    public void close() {
        if (!finished) {
            QRD.INSTANCE.writerFree(handle);
        }
    }
}