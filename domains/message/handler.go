package message

import (
	"fmt"
	"strings"
)

// Process to process Status codes
func Process(site string, code int, msg string) (output string) {
	output = fmt.Sprintf("Site: `%s`\n\n", site)
	switch {
	case code == 0 || code == 1:
		output += fmt.Sprintf("Hoppla! We have an error message 🤒\n\n`%s`", msg)
	case code >= 200 && code <= 299:
		output += fmt.Sprintf("Joohoo! It's live and kicking 🙂!\n\nStatus: [%d](https://httpstatuses.com/%d)", code, code)
	case code >= 400 && code <= 499:
		output += fmt.Sprintf("Erm! Did I do something wrong? 🤔\n\nStatus: [%d](https://httpstatuses.com/%d)", code, code)
	case code >= 500 && code <= 599:
		output += fmt.Sprintf("Schade! It's down or inaccessible to me 😟\n\nStatus: [%d](https://httpstatuses.com/%d)", code, code)
	default:
		output += "Something is fishy 🐟"
	}
	return
}

// Template to provide generic text
func Template(temp string) (output string) {
	switch temp {
	case "start", "help":
		output = "I can understand these commands:\n\n" +
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
	case "list":
		output = "Here are your tracked domains:\n\n"
	case "emptylist":
		output = "Your list is empty."
	case "about":
		output = "*ManDown*:\n\n" +
			"Open Source on [GitHub](https://github.com/Donnie/ManDown)\n" +
			"Hosted on Vultr.com in New Jersey, USA\n" +
			"No personally identifiable information is stored or used by this bot."
	default:
		output = "Didn't really get you. /help"
	}

	return
}

// ExtractMotive extracts the slash-command from a Telegram message
func ExtractMotive(text string) (string, string) {
	s := strings.Fields(text)

	if len(s) < 1 {
		return "", ""
	}
	if strings.Contains(s[0], "/about") {
		return "about", ""
	}
	if strings.Contains(s[0], "/start") {
		return "start", ""
	}
	if strings.Contains(s[0], "/help") {
		return "help", ""
	}
	if strings.Contains(s[0], "/list") {
		return "list", ""
	}
	if strings.Contains(s[0], "/clear") {
		return "clear", ""
	}

	if len(s) != 2 {
		return "", ""
	}
	if strings.Contains(s[0], "/track") {
		return "track", s[1]
	}
	if strings.Contains(s[0], "/untrack") {
		return "untrack", s[1]
	}
	return "", ""
}
