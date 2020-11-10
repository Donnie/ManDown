package main

import (
	"github.com/Donnie/ManDown/file"
	"github.com/Donnie/ManDown/web"
	tb "gopkg.in/tucnak/telebot.v2"
)

func (glob *Global) handleUntrack(m *tb.Message) {
	site := web.Sanitise(m.Payload)
	var records [][]string
	lines, _ := file.ReadCSV(glob.File)

	for _, line := range lines {
		rec := Record{}
		rec.Unmarshal(line)
		if site == rec.Site && m.Sender.ID == rec.UserID {
			continue
		}
		records = append(records, line)
	}

	file.WriteFileCSV(records, glob.File)
	glob.Bot.Send(m.Sender, "Removed!", tb.ModeMarkdown)
}
