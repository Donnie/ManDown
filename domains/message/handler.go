package message

import (
	"fmt"
	"strings"
)

// Process to process Status codes
func Process(site string, code int, msg string) string {
	var output string
	switch code {
	case 0, 1:
		output = fmt.Sprintf("Site: `%s`\n\nHoppla! We have an error message.\n\n `%s`", site, msg)
	case 200, 201:
		output = fmt.Sprintf("Site: `%s`\n\nJoohoo! It's a %d Cap'n", site, code)
	default:
		output = fmt.Sprintf("Site: `%s`\n\nSchade! It says %d", site, code)
	}
	return output
}

// Template to provide generic text
func Template(temp string) string {
	var output string
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
			"Eg: `/clear`\n\n"
	case "list":
		output = "Here are your tracked domains:\n\n"
	default:
		output = "Didn't really get you. /help"
	}

	return output
}

// ExtractMotive extracts the slash-command from a Telegram message
func ExtractMotive(text string) (string, string) {
	s := strings.Fields(text)
	if strings.Contains(s[0], "/help") {
		return "help", ""
	}
	if strings.Contains(s[0], "/start") {
		return "start", ""
	}
	if strings.Contains(s[0], "/track") {
		return "track", s[1]
	}
	if strings.Contains(s[0], "/untrack") {
		return "untrack", s[1]
	}
	if strings.Contains(s[0], "/list") {
		return "list", ""
	}
	if strings.Contains(s[0], "/clear") {
		return "clear", ""
	}
	return "", ""
}
