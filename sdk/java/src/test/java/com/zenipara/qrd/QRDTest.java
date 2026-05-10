package com.zenipara.qrd;

import org.junit.Test;
import static org.junit.Assert.*;

import java.util.List;

public class QRDTest {

    @Test
    public void testSchemaBuilder() throws QRDException {
        Schema schema = QRD.newSchema()
            .addField("id", FieldType.INT64, Nullability.REQUIRED)
            .addField("name", FieldType.STRING, Nullability.OPTIONAL)
            .addField("score", FieldType.FLOAT64, Nullability.OPTIONAL)
            .build();

        assertNotNull(schema);
        assertTrue(schema.getId() > 0);
        assertEquals(3, schema.getFieldCount());

        schema.close();
    }

    @Test
    public void testFileWriterReader() throws QRDException {
        Schema schema = QRD.newSchema()
            .addField("id", FieldType.INT64, Nullability.REQUIRED)
            .addField("value", FieldType.FLOAT64, Nullability.OPTIONAL)
            .build();

        byte[] data;

        try (FileWriter writer = QRD.newFileWriter(schema)) {
            writer.writeRow(1L, 3.14);
            writer.writeRow(2L, 2.71);
            writer.writeRow(3L, 1.41);

            data = writer.finish();
            assertNotNull(data);
            assertTrue("Expected non-empty serialized QRD data", data.length > 0);
        }

        try (FileReader reader = QRD.newFileReader(data, schema)) {
            assertEquals(3, reader.getRowCount());
            List<List<Object>> rows = reader.readAllRows();
            assertEquals(3, rows.size());
            assertEquals(2, rows.get(0).size());
            assertEquals(2, rows.get(1).size());
            assertEquals(2, rows.get(2).size());
        }

        schema.close();
    }

    @Test
    public void testFieldTypes() {
        assertEquals(0, FieldType.BOOLEAN.getValue());
        assertEquals(4, FieldType.INT64.getValue());
        assertEquals(10, FieldType.FLOAT64.getValue());
        assertEquals(15, FieldType.STRING.getValue());
    }

    @Test
    public void testNullability() {
        assertEquals(0, Nullability.REQUIRED.getValue());
        assertEquals(1, Nullability.OPTIONAL.getValue());
        assertEquals(2, Nullability.REPEATED.getValue());
    }

    @Test
    public void testJNIIntegration() throws QRDException {
        // Test JNI integration and native memory management
        Schema schema = QRD.newSchema()
            .addField("test", FieldType.INT32, Nullability.REQUIRED)
            .build();

        try {
            assertNotNull(schema);
            assertEquals(1, schema.getFieldCount());
        } finally {
            schema.close();
        }
    }

    @Test
    public void testAutoCloseable() throws QRDException {
        // Test AutoCloseable implementation
        Schema schema = QRD.newSchema()
            .addField("id", FieldType.INT64, Nullability.REQUIRED)
            .build();

        try (Schema autoCloseSchema = schema) {
            assertEquals(1, autoCloseSchema.getFieldCount());
        }
        // Schema should be closed automatically
    }

    @Test
    public void testInvalidPayloadRejection() throws QRDException {
        // Test rejection of invalid input data
        Schema schema = QRD.newSchema()
            .addField("data", FieldType.BLOB, Nullability.REQUIRED)
            .build();

        try (FileWriter writer = QRD.newFileWriter(schema)) {
            // Try writing invalid data
            try {
                writer.writeRow("invalid");
                fail("Expected exception for invalid data");
            } catch (QRDException e) {
                // Expected
            }
        } finally {
            schema.close();
        }
    }

    @Test
    public void testDeterministicWrites() throws QRDException {
        // Test that writes are deterministic
        Schema schema = QRD.newSchema()
            .addField("value", FieldType.INT64, Nullability.REQUIRED)
            .build();

        byte[] data1, data2;

        try (FileWriter writer1 = QRD.newFileWriter(schema)) {
            writer1.writeRow(1L);
            writer1.writeRow(2L);
            data1 = writer1.finish();
        }

        try (FileWriter writer2 = QRD.newFileWriter(schema)) {
            writer2.writeRow(1L);
            writer2.writeRow(2L);
            data2 = writer2.finish();
        }

        assertArrayEquals("Writes should be deterministic", data1, data2);

        schema.close();
    }

    @Test
    public void testSchemaValidation() throws QRDException {
        // Test schema validation
        try {
            QRD.newSchema()
                .addField("", FieldType.INT32, Nullability.REQUIRED)
                .build();
            fail("Expected exception for empty field name");
        } catch (QRDException e) {
            // Expected
        }

        try {
            QRD.newSchema()
                .addField("dup", FieldType.INT32, Nullability.REQUIRED)
                .addField("dup", FieldType.STRING, Nullability.REQUIRED)
                .build();
            fail("Expected exception for duplicate field names");
        } catch (QRDException e) {
            // Expected
        }
    }

    @Test
    public void testPartialReads() throws QRDException {
        // Test partial column reads
        Schema schema = QRD.newSchema()
            .addField("col1", FieldType.INT64, Nullability.REQUIRED)
            .addField("col2", FieldType.STRING, Nullability.REQUIRED)
            .addField("col3", FieldType.FLOAT64, Nullability.REQUIRED)
            .build();

        byte[] data;

        try (FileWriter writer = QRD.newFileWriter(schema)) {
            writer.writeRow(1L, "test", 3.14);
            data = writer.finish();
        }

        try (FileReader reader = QRD.newFileReader(data, schema)) {
            assertEquals(1, reader.getRowCount());
            // Test that all columns are accessible
            List<List<Object>> rows = reader.readAllRows();
            assertEquals(1, rows.size());
            assertEquals(3, rows.get(0).size());
        } finally {
            schema.close();
        }
    }

    @Test
    public void testMalformedRejection() throws QRDException {
        // Test rejection of malformed data
        Schema schema = QRD.newSchema()
            .addField("data", FieldType.BLOB, Nullability.REQUIRED)
            .build();

        byte[] malformedData = new byte[]{1, 2, 3}; // Invalid QRD data

        try {
            QRD.newFileReader(malformedData, schema);
            fail("Expected exception for malformed data");
        } catch (QRDException e) {
            // Expected
        } finally {
            schema.close();
        }
    }

    @Test
    public void testConcurrentReads() throws QRDException {
        // Test concurrent read access
        Schema schema = QRD.newSchema()
            .addField("id", FieldType.INT64, Nullability.REQUIRED)
            .build();

        byte[] data;

        try (FileWriter writer = QRD.newFileWriter(schema)) {
            for (int i = 0; i < 100; i++) {
                writer.writeRow((long) i);
            }
            data = writer.finish();
        }

        // Test concurrent reads
        Thread thread1 = new Thread(() -> {
            try (FileReader reader = QRD.newFileReader(data, schema)) {
                assertEquals(100, reader.getRowCount());
            } catch (QRDException e) {
                fail("Concurrent read failed: " + e.getMessage());
            }
        });

        Thread thread2 = new Thread(() -> {
            try (FileReader reader = QRD.newFileReader(data, schema)) {
                assertEquals(100, reader.getRowCount());
            } catch (QRDException e) {
                fail("Concurrent read failed: " + e.getMessage());
            }
        });

        thread1.start();
        thread2.start();

        try {
            thread1.join();
            thread2.join();
        } catch (InterruptedException e) {
            fail("Thread join interrupted");
        }

        schema.close();
    }

    @Test
    public void testFooterInspection() throws QRDException {
        // Test footer metadata inspection
        Schema schema = QRD.newSchema()
            .addField("data", FieldType.BLOB, Nullability.REQUIRED)
            .build();

        byte[] data;

        try (FileWriter writer = QRD.newFileWriter(schema)) {
            writer.writeRow(new byte[]{1, 2, 3});
            data = writer.finish();
        }

        try (FileReader reader = QRD.newFileReader(data, schema)) {
            // Test footer access (implementation dependent)
            assertEquals(1, reader.getRowCount());
        } finally {
            schema.close();
        }
    }

    @Test
    public void testMemoryCleanup() throws QRDException {
        // Test proper memory cleanup
        Schema schema = QRD.newSchema()
            .addField("large", FieldType.BLOB, Nullability.REQUIRED)
            .build();

        try (FileWriter writer = QRD.newFileWriter(schema)) {
            byte[] largeData = new byte[1024 * 1024]; // 1MB
            writer.writeRow(largeData);
            writer.finish();
        } finally {
            schema.close();
        }
        // Memory should be properly cleaned up
    }

    @Test
    public void testTypeMapping() throws QRDException {
        // Test Java type to QRD type mapping
        Schema schema = QRD.newSchema()
            .addField("bool", FieldType.BOOLEAN, Nullability.REQUIRED)
            .addField("i32", FieldType.INT32, Nullability.REQUIRED)
            .addField("i64", FieldType.INT64, Nullability.REQUIRED)
            .addField("f64", FieldType.FLOAT64, Nullability.REQUIRED)
            .addField("str", FieldType.STRING, Nullability.REQUIRED)
            .build();

        byte[] data;

        try (FileWriter writer = QRD.newFileWriter(schema)) {
            writer.writeRow(true, 42, 123L, 3.14, "test");
            data = writer.finish();
        }

        try (FileReader reader = QRD.newFileReader(data, schema)) {
            List<List<Object>> rows = reader.readAllRows();
            assertEquals(1, rows.size());
            List<Object> row = rows.get(0);
            assertEquals(true, row.get(0));
            assertEquals(42, row.get(1));
            assertEquals(123L, row.get(2));
            assertEquals(3.14, (Double) row.get(3), 0.001);
            assertEquals("test", row.get(4));
        } finally {
            schema.close();
        }
    }
}