package main

import (
    "fmt"
    "os"
    "encoding/binary"
    "io/ioutil"
    qrd "github.com/zenipara/QRD-SDK/sdk/go"
)

func encodeString(s string) []byte {
    b := []byte(s)
    out := make([]byte, 4+len(b))
    binary.LittleEndian.PutUint32(out[:4], uint32(len(b)))
    copy(out[4:], b)
    return out
}

func main() {
    path := "/tmp/cross_go.qrd"
    if len(os.Args) > 1 {
        path = os.Args[1]
    }

    // build schema
    sb := qrd.NewSchemaBuilder()
    sb.AddField("id", qrd.FieldTypeInt64, qrd.NullabilityRequired, "")
    sb.AddField("name", qrd.FieldTypeString, qrd.NullabilityRequired, "")
    schema, err := sb.Build()
    if err != nil {
        fmt.Printf("build schema err: %v\n", err)
        os.Exit(1)
    }

    w, err := qrd.NewFileWriter(schema)
    if err != nil {
        fmt.Printf("writer err: %v\n", err)
        os.Exit(1)
    }

    for i := 0; i < 50; i++ {
        id := int64(i)
        name := fmt.Sprintf("gouser_%d", i)
        if err := w.WriteRow([]interface{}{id, encodeString(name)}); err != nil {
            fmt.Printf("write row err: %v\n", err)
            os.Exit(1)
        }
    }

    data, err := w.Finish()
    if err != nil {
        fmt.Printf("finish err: %v\n", err)
        os.Exit(1)
    }

    if err := ioutil.WriteFile(path, data, 0644); err != nil {
        fmt.Printf("write file err: %v\n", err)
        os.Exit(1)
    }
    fmt.Printf("Wrote %d bytes to %s\n", len(data), path)
}

