package main

import (
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
	go glob.Bot.Send(m.Sender, output, tb.ModeMarkdown)

	if result.Status != 0 && result.Status != 1 {
		rec := Record{
			Site:      site,
			UserID:    m.Sender.ID,
			MessageID: m.ID,
			Time:      time.Now(),
			Status:    result.Status,
		}

		lines, _ := file.ReadCSV(glob.File)
		if !rec.ExistsIn(lines) {
			file.WriteLineCSV(rec.Marshall(), glob.File)
		}
	}
}
