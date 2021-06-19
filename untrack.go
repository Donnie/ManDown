package main

import (
	"github.com/Donnie/ManDown/message"
	"github.com/Donnie/ManDown/web"
	tb "gopkg.in/tucnak/telebot.v2"
)

func (glob *Global) handleUntrack(m *tb.Message) {
	var output string
	plain, ssl, err := web.Sanitise(m.Payload)
	if err != nil {
		output = message.InputError(err)
		go glob.Bot.Send(m.Sender, output, tb.ModeMarkdown)
		return
	}
	if plain != "" {
		glob.handleRemove(plain, m)
	}
	if ssl != "" {
		glob.handleRemove(ssl, m)
	}
}

func (glob *Global) handleRemove(site string, m *tb.Message) {
	rec := Record{
		Site:   site,
		UserID: m.Sender.ID,
	}
	rec.Delete(glob.File)
	glob.Bot.Send(m.Sender, "Removed!", tb.ModeMarkdown)
}
