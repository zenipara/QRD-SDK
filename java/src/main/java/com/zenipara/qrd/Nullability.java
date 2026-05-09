package com.zenipara.qrd;

/**
 * Field nullability options
 */
public enum Nullability {
    REQUIRED(0),
    OPTIONAL(1),
    REPEATED(2);

    private final int value;

    Nullability(int value) {
        this.value = value;
    }

    public int getValue() {
        return value;
    }
}