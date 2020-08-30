package main

import tb "gopkg.in/tucnak/telebot.v2"

func (gl *Global) handleAbout(m *tb.Message) {
	output := "*ManDown*:\n\n" +
		"Open Source on [GitHub](https://github.com/Donnie/ManDown)\n" +
		"Hosted on Vultr.com in New Jersey, USA\n" +
		"No personally identifiable information is stored or used by this bot."

	gl.Bot.Send(m.Sender, output)
}
