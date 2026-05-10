package com.zenipara.qrd;

import com.sun.jna.Native;
import com.sun.jna.Pointer;
import com.sun.jna.ptr.LongByReference;
import com.sun.jna.ptr.PointerByReference;
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

        long rowHandle = QRD.INSTANCE.qrd_row_new();
        if (rowHandle == 0) {
            throw new QRDException("Failed to create row");
        }

        try {
            for (Object value : values) {
                addValueToRow(rowHandle, value);
            }

            int result = QRD.INSTANCE.qrd_writer_write_row(handle, rowHandle);
            if (result != 0) {
                throw new QRDException("Failed to write row");
            }
        } finally {
            QRD.INSTANCE.qrd_row_free(rowHandle);
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

        PointerByReference dataRef = new PointerByReference();
        LongByReference sizeRef = new LongByReference();

        int result = QRD.INSTANCE.qrd_writer_finish(handle, dataRef, sizeRef);
        if (result != 0) {
            throw new QRDException("Failed to finish writer");
        }

        Pointer dataPtr = dataRef.getValue();
        long size = sizeRef.getValue();
        if (size == 0 || dataPtr == null) {
            finished = true;
            return new byte[0];
        }

        if (size > Integer.MAX_VALUE) {
            throw new QRDException("Output is too large to fit in a Java byte array");
        }

        byte[] bytes = dataPtr.getByteArray(0, (int) size);
        Native.free(Pointer.nativeValue(dataPtr));
        finished = true;
        return bytes;
    }

    private void addValueToRow(long rowHandle, Object value) throws QRDException {
        if (value == null) {
            // Handle null values - would need proper implementation
            return;
        }

        if (value instanceof Long) {
            QRD.INSTANCE.qrd_row_add_int64(rowHandle, (Long) value);
        } else if (value instanceof Double) {
            QRD.INSTANCE.qrd_row_add_float64(rowHandle, (Double) value);
        } else if (value instanceof String) {
            QRD.INSTANCE.qrd_row_add_string(rowHandle, (String) value);
        } else if (value instanceof Integer) {
            QRD.INSTANCE.qrd_row_add_int64(rowHandle, ((Integer) value).longValue());
        } else if (value instanceof Float) {
            QRD.INSTANCE.qrd_row_add_float64(rowHandle, ((Float) value).doubleValue());
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
            QRD.INSTANCE.qrd_writer_free(handle);
        }
    }
}