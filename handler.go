package main

import (
	"strconv"
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
	var records []Record
	for _, line := range lines {
		record := &Record{}
		records = append(records, record.Unmarshall(line))
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
				if result.Status != int(rec.Status) {
					rec.Status = result.Status
					rec.Time = time.Now()
					output := message.Process(result.Site, result.Status, result.Misc)
					go glob.Bot.Send(&tb.User{ID: int(rec.UserID)}, output, tb.ModeMarkdown)
				}
				linesOut = append(linesOut, rec.Marshall())
			}
		}
	}
	return
}

// Unmarshall string to record
func (rec *Record) Unmarshall(lineIn []string) (rx Record) {
	rx.Site = lineIn[0]
	rx.UserID, _ = strconv.Atoi(lineIn[1])
	rx.MessageID, _ = strconv.Atoi(lineIn[2])
	rx.Time, _ = time.Parse(layout, lineIn[3])
	rx.Status, _ = strconv.Atoi(lineIn[4])
	return
}

// Marshall to strings
func (rec *Record) Marshall() []string {
	return []string{
		rec.Site,
		strconv.Itoa(rec.UserID),
		strconv.Itoa((int(rec.MessageID))),
		rec.Time.Format(layout),
		strconv.Itoa(int(rec.Status)),
	}
}
