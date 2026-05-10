package com.zenipara.qrd;

import java.util.ArrayList;
import java.util.List;

/**
 * Builder for creating QRD schemas
 */
public class SchemaBuilder {
    private final List<Field> fields = new ArrayList<>();

    /**
     * Add a field to the schema
     */
    public SchemaBuilder addField(String name, FieldType fieldType, Nullability nullability) {
        return addField(name, fieldType, nullability, null);
    }

    /**
     * Add a field to the schema with metadata
     */
    public SchemaBuilder addField(String name, FieldType fieldType, Nullability nullability, String metadata) {
        fields.add(new Field(name, fieldType, nullability, metadata));
        return this;
    }

    /**
     * Build the schema
     */
    public Schema build() throws QRDException {
        long handle = QRD.INSTANCE.schemaNew();
        if (handle == 0) {
            throw new QRDException("Failed to create schema");
        }

        try {
            for (Field field : fields) {
                int result = QRD.INSTANCE.schemaAddField(handle,
                    field.getName(),
                    field.getFieldType().getValue(),
                    field.getNullability().getValue(),
                    field.getMetadata());
                if (result != 0) {
                    throw new QRDException("Failed to add field: " + field.getName());
                }
            }

            long id = QRD.INSTANCE.schemaId(handle);
            return new Schema(handle, id, new ArrayList<>(fields));
        } catch (Exception e) {
