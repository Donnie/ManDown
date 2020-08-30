package main

import (
	"strconv"

	"github.com/Donnie/ManDown/file"
	"github.com/Donnie/ManDown/message"
	tb "gopkg.in/tucnak/telebot.v2"
)

func (glob *Global) handleList(m *tb.Message) {
	var records [][]string
	var output string
	lines, _ := file.ReadCSV(glob.File)

	for _, line := range lines {
		chatID, _ := strconv.Atoi(line[1])

		if chatID != m.Sender.ID {
			continue
		}
		records = append(records, line)
	}

	if len(records) == 0 {
		output = message.Template("emptylist")
	} else {
		output = message.Template("list")
		for num, record := range records {
			output = output + strconv.Itoa(num+1) + ". `" + record[0] + "`\n"
		}
	}
	glob.Bot.Send(m.Sender, output, tb.ModeMarkdown)
}
