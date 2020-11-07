package main

import (
	"strconv"
	"strings"
	"time"

	"github.com/Donnie/ManDown/file"
	"github.com/Donnie/ManDown/message"
	"github.com/Donnie/ManDown/web"
	tb "gopkg.in/tucnak/telebot.v2"
)

var layout = "2006-01-02 15:04:05"

func (glob *Global) poll(freq string) {
	i, _ := strconv.Atoi(freq)
	for range time.Tick(time.Second * time.Duration(i)) {
		glob.executePoll()
	}
}

func (glob *Global) executePoll() {
	lines, _ := file.ReadCSV(glob.File)

	records := make([]Record, len(lines))
	for i, line := range lines {
		records[i].Unmarshall(line)
	}

	linesOut := glob.handleRecords(records)
	file.WriteFileCSV(linesOut, glob.File)
}

func (glob *Global) handleRecords(recs []Record) (linesOut [][]string) {
	var sites []string
	for _, rec := range recs {
		sites = append(sites, rec.Site)
	}
	results := web.CheckBulk(sites)

	for _, rec := range recs {
		for _, result := range results {
			if result.Site == rec.Site {
				if result.Status != rec.Status {
					// ignore transport layer errors
					if strings.Contains(result.Misc, "tcp") {
						continue
					}
					rec.Status = result.Status
					rec.Time = time.Now()
					output := message.Process(result.Site, result.Status, result.Misc)
					go glob.Bot.Send(&tb.User{ID: rec.UserID}, output, tb.ModeMarkdown)
				}
				linesOut = append(linesOut, rec.Marshall())
			}
		}
	}
	return
}

// Unmarshall string to record
func (rec *Record) Unmarshall(lineIn []string) {
	rec.Site = lineIn[0]
	rec.UserID, _ = strconv.Atoi(lineIn[1])
	rec.MessageID, _ = strconv.Atoi(lineIn[2])
	rec.Time, _ = time.Parse(layout, lineIn[3])
	rec.Status, _ = strconv.Atoi(lineIn[4])
}

// Marshall to strings
func (rec *Record) Marshall() []string {
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
		exist.Unmarshall(line)
		if exist.Site == rec.Site && exist.UserID == rec.UserID {
			return true
		}
	}
	return false
}
