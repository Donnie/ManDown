package main

import (
	"strconv"

	"github.com/Donnie/ManDown/file"
	tb "gopkg.in/tucnak/telebot.v2"
)

func (glob *Global) handleClear(m *tb.Message) {
	var records [][]string
	lines, _ := file.ReadCSV(glob.File)

	for _, line := range lines {
		chatID, _ := strconv.Atoi(line[1])

		if chatID == m.Sender.ID {
			continue
		}
		records = append(records, line)
	}

	file.WriteFileCSV(records, glob.File)
	glob.Bot.Send(m.Sender, "All Clear!", tb.ModeMarkdown)
}
