package com.zenipara.qrd;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;
import com.sun.jna.ptr.IntByReference;
import com.sun.jna.ptr.LongByReference;
import com.sun.jna.ptr.PointerByReference;

/**
 * Main QRD class providing access to QRD functionality
 */
public class QRD {
    public static final QRDInterface INSTANCE = Native.load("qrd_ffi", QRDInterface.class);

    /**
     * Create a new schema builder
     */
    public static SchemaBuilder newSchema() {
        return new SchemaBuilder();
    }

    /**
     * Create a new file writer
     */
    public static FileWriter newFileWriter(Schema schema) throws QRDException {
        long handle = INSTANCE.writerNew(schema.getHandle());
        if (handle == 0) {
            throw new QRDException("Failed to create writer");
        }
        return new FileWriter(handle);
    }

    /**
     * Create a new file reader
     */
    public static FileReader newFileReader(byte[] data) throws QRDException {
        long handle = INSTANCE.readerNew(data, data.length);
        if (handle == 0) {
            throw new QRDException("Failed to create reader");
        }

        long schemaHandle = INSTANCE.readerSchema(handle);
        if (schemaHandle == 0) {
            INSTANCE.readerFree(handle);
            throw new QRDException("Failed to get schema from reader");
        }

        // Create schema object (simplified - would need proper field extraction)
        Schema schema = new Schema(schemaHandle, INSTANCE.schemaId(schemaHandle), java.util.Collections.emptyList());

        return new FileReader(handle, schema);
    }

    /**
     * Create a new file reader with a known schema.
     */
    public static FileReader newFileReader(byte[] data, Schema schema) throws QRDException {
        long handle = INSTANCE.readerNew(data, data.length);
        if (handle == 0) {
            throw new QRDException("Failed to create reader");
        }
        return new FileReader(handle, schema);
    }

    /**
     * JNA interface to the native QRD library
     */
    public interface QRDInterface extends Library {
        // Schema functions
        long schemaNew();
        void schemaFree(long schema);
        int schemaAddField(long schema, String name, int fieldType, int nullability, String metadata);
        long schemaId(long schema);
        int schemaFieldCount(long schema);

        // Writer functions
        long writerNew(long schema);
        void writerFree(long writer);
        int writerWriteRow(long writer, long row);
        int writerFinish(long writer, PointerByReference data, LongByReference size);

        // Reader functions
        long readerNew(byte[] data, int size);
        void readerFree(long reader);
        long readerSchema(long reader);
        long readerRowCount(long reader);
        long readerReadRow(long reader);

        // Row functions
        long rowNew();
        void rowFree(long row);
        int rowFieldCount(long row);
        Pointer rowGetBytes(long row, int index, IntByReference size);
        int rowAddInt64(long row, long value);
        int rowAddFloat64(long row, double value);
        int rowAddString(long row, String value);
    }
}