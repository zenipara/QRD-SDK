package com.zenipara.qrd;

/**
 * Exception thrown by QRD operations
 */
public class QRDException extends Exception {
    public QRDException(String message) {
        super(message);
    }

    public QRDException(String message, Throwable cause) {
        super(message, cause);
    }
}