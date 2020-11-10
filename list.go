package main

import (
	"fmt"

	"github.com/Donnie/ManDown/file"
	"github.com/Donnie/ManDown/message"
	tb "gopkg.in/tucnak/telebot.v2"
)

func (glob *Global) handleList(m *tb.Message) {
	var records []Record
	lines, _ := file.ReadCSV(glob.File)

	for _, line := range lines {
		rec := Record{}
		rec.Unmarshal(line)
		if rec.UserID != m.Sender.ID {
			continue
		}
		records = append(records, rec)
	}

	output := message.Template("emptylist")
	if len(records) != 0 {
		output = message.Template("list")
		for num, record := range records {
			output += fmt.Sprintf("%d. `%s`\n", num+1, record.Site)
		}
	}
	glob.Bot.Send(m.Sender, output, tb.ModeMarkdown)
}
