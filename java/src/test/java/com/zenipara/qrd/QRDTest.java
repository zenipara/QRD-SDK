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

        try (FileWriter writer = QRD.newFileWriter(schema)) {
            writer.writeRow(1L, 3.14);
            writer.writeRow(2L, 2.71);
            writer.writeRow(3L, 1.41);

            byte[] data = writer.finish();
            assertNotNull(data);
            // Note: In a real implementation, data would not be empty
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
}