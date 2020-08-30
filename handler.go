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
	var records [][]string
	var sites []string

	lines, _ := file.ReadCSV(glob.File)

	for _, line := range lines {
		sites = append(sites, line[0])
	}
	results := web.CheckBulk(sites)

	for _, line := range lines {
		site := line[0]
		chatID, _ := strconv.Atoi(line[1])
		// msgID, _ := strconv.ParseInt(line[2], 10, 64)
		tyme, _ := time.Parse(layout, line[3])
		status, _ := strconv.Atoi(line[4])

		for _, result := range results {
			if result.Site == site {
				if result.Status != status {
					tyme = time.Now()
					output := message.Process(result.Site, result.Status, result.Misc)
					go glob.Bot.Send(&tb.User{ID: chatID}, output, tb.ModeMarkdown)
				}
				record := []string{
					site,
					line[1],
					line[2],
					tyme.Format(layout),
					strconv.Itoa(result.Status),
				}
				records = append(records, record)
			}
		}
	}

	file.WriteFileCSV(records, glob.File)
}
