package main

import (
	"time"

	"github.com/Donnie/ManDown/message"
	"github.com/Donnie/ManDown/web"
	tb "gopkg.in/tucnak/telebot.v2"
)

func (glob *Global) handleTrack(m *tb.Message) {
	var output string
	plain, ssl, err := web.Sanitise(m.Payload)
	if err != nil {
		output = message.InputError(err)
		go glob.Bot.Send(m.Sender, output, tb.ModeMarkdown)
		return
	}
	if plain != "" {
		glob.handleAdd(plain, m)
	}
	if ssl != "" {
		glob.handleAdd(ssl, m)
	}
}

func (glob *Global) handleAdd(site string, m *tb.Message) {
	check := web.CheckHealth(site)
	output := message.Process(check.Site, check.Status, check.Misc)
	go glob.Bot.Send(m.Sender, output, tb.ModeMarkdown)

	if check.IsAccepted() {
		rec := Record{
			Site:      site,
			UserID:    m.Sender.ID,
			MessageID: m.ID,
			Time:      time.Now(),
			Status:    check.Status,
		}
		rec.Put(glob.File)
	}
}
