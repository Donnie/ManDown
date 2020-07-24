package file

import (
	"bufio"
	"encoding/csv"
	"io"
	"log"
	"os"
	"strings"

	"github.com/pkg/errors"
)

func WriteFileCSV(records [][]string, file string) error {
	err := os.Remove(file)
	if err != nil {
		return errors.Wrap(err, "file deletion failed")
	}

	createPath(file)
	csvFile, err := os.OpenFile(file, os.O_CREATE|os.O_WRONLY, 0644)
	if err != nil {
		return errors.Wrap(err, "file creation failed")
	}
	defer csvFile.Close()

	w := csv.NewWriter(csvFile)
	if err := w.WriteAll(records); err != nil {
		return errors.Wrap(err, "error writing record")
	}

	return nil
}

func WriteLineCSV(record []string, file string) error {
	createPath(file)
	csvFile, err := os.OpenFile(file, os.O_CREATE|os.O_APPEND|os.O_WRONLY, 0644)
	if err != nil {
		return errors.Wrap(err, "file open failed")
	}
	defer csvFile.Close()

	w := csv.NewWriter(csvFile)
	if err := w.Write(record); err != nil {
		return errors.Wrap(err, "error writing record")
	}
	w.Flush()

	return nil
}

func ReadCSV(file string) ([][]string, error) {
	createPath(file)
	csvFile, err := os.Open(file)
	if err != nil {
		return nil, errors.Wrap(err, "file open failed")
	}
	defer csvFile.Close()

	reader := csv.NewReader(bufio.NewReader(csvFile))
	var lines [][]string

	for {
		line, error := reader.Read()
		if error == io.EOF {
			break
		} else if error != nil {
			log.Fatal(error)
		}
		lines = append(lines, line)
	}

	return lines, nil
}

func createPath(file string) {
	if _, err := os.Stat(file); os.IsNotExist(err) {
		buff := strings.Split(file, "/")
		filedir := strings.Join(buff[:len(buff)-1], "/")
		os.MkdirAll(filedir, 0700)
	}
}
