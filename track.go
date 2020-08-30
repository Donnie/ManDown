package main

import (
	"strconv"
	"time"

	"github.com/Donnie/ManDown/file"
	"github.com/Donnie/ManDown/message"
	"github.com/Donnie/ManDown/web"
	tb "gopkg.in/tucnak/telebot.v2"
)

func (glob *Global) handleTrack(m *tb.Message) {
	site := web.Sanitise(m.Payload)
	result := web.CheckHealth(site)
	output := message.Process(result.Site, result.Status, result.Misc)

	if result.Status != 0 && result.Status != 1 {
		lines, _ := file.ReadCSV(glob.File)
		for _, line := range lines {
			chatID, _ := strconv.Atoi(line[1])
			if site == line[0] && chatID == m.Sender.ID {
				glob.Bot.Send(m.Sender, output, tb.ModeMarkdown)
				return
			}
		}

		tyme := time.Now()
		record := []string{
			site,
			strconv.Itoa(m.Sender.ID),
			strconv.Itoa(m.ID),
			tyme.Format(layout),
			strconv.Itoa(result.Status),
		}

		file.WriteLineCSV(record, glob.File)
	}

	glob.Bot.Send(m.Sender, output, tb.ModeMarkdown)
}
