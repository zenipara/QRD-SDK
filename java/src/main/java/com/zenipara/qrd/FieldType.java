package com.zenipara.qrd;

/**
 * Field types supported by QRD
 */
public enum FieldType {
    BOOLEAN(0),
    INT8(1),
    INT16(2),
    INT32(3),
    INT64(4),
    UINT8(5),
    UINT16(6),
    UINT32(7),
    UINT64(8),
    FLOAT32(9),
    FLOAT64(10),
    TIMESTAMP(11),
    DATE(12),
    TIME(13),
    DURATION(14),
    STRING(15),
    ENUM(16),
    UUID(17),
    BLOB(18),
    DECIMAL(19);

    private final int value;

    FieldType(int value) {
        this.value = value;
    }

    public int getValue() {
        return value;
    }
}