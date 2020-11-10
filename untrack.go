package main

import (
	"github.com/Donnie/ManDown/web"
	tb "gopkg.in/tucnak/telebot.v2"
)

func (glob *Global) handleUntrack(m *tb.Message) {
	rec := Record{
		Site:   web.Sanitise(m.Payload),
		UserID: m.Sender.ID,
	}
	rec.Delete(glob.File)
	glob.Bot.Send(m.Sender, "Removed!", tb.ModeMarkdown)
}
