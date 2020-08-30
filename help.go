package main

import (
	"fmt"

	tb "gopkg.in/tucnak/telebot.v2"
)

func (gl *Global) handleHelp(m *tb.Message) {
	output := fmt.Sprintf("Hello %s!\n\n", m.Sender.FirstName)
	output += "I can understand these commands:\n\n" +
		"`/track yourdomain.com` - Get notified when the status of your domain changes\n" +
		"Eg: `/track telegram.org`\n\n" +
		"`/untrack yourdomain.com` - Stop following a domain\n" +
		"Eg: `/untrack telegram.org`\n\n" +
		"/list - Get a list of your followed domains\n" +
		"Eg: `/list`\n\n" +
		"/clear - Clear your list of your followed domains\n" +
		"Eg: `/clear`\n\n" +
		"/about - Read About\n" +
		"Eg: `/about`\n\n"

	gl.Bot.Send(m.Sender, output)
}
