package qrd

import (
    "io/ioutil"
    "testing"
)

func TestCrossReadRustFile(t *testing.T) {
    data, err := ioutil.ReadFile("/tmp/cross_rust.qrd")
    if err != nil {
        t.Fatalf("failed to read file: %v", err)
    }

    reader, err := NewFileReader(data)
    if err != nil {
        t.Fatalf("failed to create reader: %v", err)
    }

    if reader.RowCount() != 100 {
        t.Fatalf("expected 100 rows, got %d", reader.RowCount())
    }
}

func TestCrossReadPythonFile(t *testing.T) {
    data, err := ioutil.ReadFile("/tmp/cross_py.qrd")
    if err != nil {
        t.Fatalf("failed to read file: %v", err)
    }

    reader, err := NewFileReader(data)
    if err != nil {
        t.Fatalf("failed to create reader: %v", err)
    }

    if reader.RowCount() != 75 {
        t.Fatalf("expected 75 rows, got %d", reader.RowCount())
    }
}

