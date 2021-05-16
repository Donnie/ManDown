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

// Global holds app state
type Global struct {
	Bot  *tb.Bot
	File string
}

func (glob *Global) poll(freq string) {
	i, _ := strconv.Atoi(freq)
	for range time.Tick(time.Second * time.Duration(i)) {
		glob.executePoll()
	}
}

func (glob *Global) executePoll() {
	if !glob.handleFaangCheck() {
		// if internet not up then do nothing
		return
	}

	lines, _ := file.ReadCSV(glob.File)

	records := make([]Record, len(lines))
	for i, line := range lines {
		records[i].Unmarshal(line)
	}

	updated := glob.handleRecords(records)
	file.WriteFileCSV(updated, glob.File)
}

func (glob *Global) handleRecords(recs []Record) (updated [][]string) {
	var sites []string
	for _, rec := range recs {
		sites = append(sites, rec.Site)
	}
	results := web.CheckBulk(sites)

	for _, rec := range recs {
		for _, result := range results {
			// ignore transport layer errors
			if strings.Contains(result.Misc, "read udp") {
				continue
			}
			if result.Site == rec.Site {
				if result.Status != rec.Status {
					// if same site but different status
					rec.Status = result.Status
					rec.Time = time.Now()
					output := message.Process(result.Site, result.Status, result.Misc)
					go glob.Bot.Send(&tb.User{ID: rec.UserID}, output, tb.ModeMarkdown)
				}
				// skip other results if found
				break
			}
		}
		updated = append(updated, rec.Marshal())
	}
	return
}

func (glob *Global) handleFaangCheck() (up bool) {
	// add FAANG
	faang := []string{
		"https://facebook.com",
		"https://apple.com",
		"https://amazon.com",
		"https://netflix.com",
		"https://google.com",
	}
	results := web.CheckBulk(faang)

	// do FAANG check
	for _, res := range results {
		// even if one is online then internet is up
		if res.Status == 200 {
			up = true
		}
	}
	return
}
