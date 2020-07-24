package file

import (
	"bufio"
	"encoding/csv"
	"fmt"
	"io"
	"log"
	"os"
)

func WriteFileCSV(records [][]string, file string) {
	os.Remove(file)
	csvFile, err := os.OpenFile(file, os.O_CREATE|os.O_WRONLY, 0644)
	if err != nil {
		fmt.Println(err)
	}
	defer csvFile.Close()
	w := csv.NewWriter(csvFile)

	if err := w.WriteAll(records); err != nil {
		log.Fatalln("error writing record to csv:", err)
	}
}

func WriteLineCSV(record []string, file string) {
	csvFile, err := os.OpenFile(file, os.O_CREATE|os.O_APPEND|os.O_WRONLY, 0644)
	if err != nil {
		fmt.Println(err)
	}
	defer csvFile.Close()
	w := csv.NewWriter(csvFile)

	if err := w.Write(record); err != nil {
		log.Fatalln("error writing record to csv:", err)
	}
	w.Flush()
}

func ReadCSV(file string) [][]string {
	csvFile, err := os.Open("db.csv")
	if err != nil {
		fmt.Println(err)
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

	return lines
}
