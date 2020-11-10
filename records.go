package main

import (
	"strconv"
	"time"

	"github.com/Donnie/ManDown/file"
)

// Record struct
type Record struct {
	Site      string
	UserID    int
	MessageID int
	Time      time.Time
	Status    int
}

var layout = "2006-01-02 15:04:05"

// Unmarshal string to record
func (rec *Record) Unmarshal(lineIn []string) {
	rec.Site = lineIn[0]
	rec.UserID, _ = strconv.Atoi(lineIn[1])
	rec.MessageID, _ = strconv.Atoi(lineIn[2])
	rec.Time, _ = time.Parse(layout, lineIn[3])
	rec.Status, _ = strconv.Atoi(lineIn[4])
}

// Marshal to strings
func (rec *Record) Marshal() []string {
	return []string{
		rec.Site,
		strconv.Itoa(rec.UserID),
		strconv.Itoa(rec.MessageID),
		rec.Time.Format(layout),
		strconv.Itoa(rec.Status),
	}
}

// ExistsIn if it already exists
func (rec *Record) ExistsIn(lines [][]string) bool {
	for _, line := range lines {
		exist := Record{}
		exist.Unmarshal(line)
		if exist.Site == rec.Site && exist.UserID == rec.UserID {
			return true
		}
	}
	return false
}

// Put if it does not already exists
func (rec *Record) Put(filename string) error {
	lines, _ := file.ReadCSV(filename)
	if rec.ExistsIn(lines) {
		return nil
	}
	return file.WriteLineCSV(rec.Marshal(), filename)
}

// Delete deletes from file
func (rec *Record) Delete(filename string) error {
	lines, _ := file.ReadCSV(filename)
	var out [][]string
	for _, line := range lines {
		if line[0] == rec.Site &&
			line[1] == strconv.Itoa(rec.UserID) {
			continue
		}
		out = append(out, line)
	}
	return file.WriteFileCSV(out, filename)
}
