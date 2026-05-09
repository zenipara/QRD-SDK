package com.zenipara.qrd;

/**
 * Schema field definition
 */
public class Field {
    private final String name;
    private final FieldType fieldType;
    private final Nullability nullability;
    private final String metadata;

    public Field(String name, FieldType fieldType, Nullability nullability, String metadata) {
        this.name = name;
        this.fieldType = fieldType;
        this.nullability = nullability;
        this.metadata = metadata;
    }

    public String getName() {
        return name;
    }

    public FieldType getFieldType() {
        return fieldType;
    }

    public Nullability getNullability() {
        return nullability;
    }

    public String getMetadata() {
        return metadata;
    }

    @Override
    public String toString() {
        return String.format("Field{name='%s', type=%s, nullability=%s}",
                           name, fieldType, nullability);
    }
}