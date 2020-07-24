package message

import (
	"fmt"
	"strings"
)

func Process(code int) string {
	var output string
	switch code {
	case 0:
		output = "Gimme a correct URL"
	case 1:
		output = "Seems like never existed"
	case 200, 201:
		output = fmt.Sprintf("It's a %d Cap'n", code)
	default:
		output = fmt.Sprintf("Bad news, it says %d", code)
	}
	return output
}

func Template(temp string) string {
	var output string
	switch temp {
	case "start", "help":
		output = "I can understand these commands:\n\n" +
			"`/track yourdomain.com` - Get notified when the status of your domain changes\n" +
			"Eg: `/track telegram.org`\n\n" +
			"`/untrack yourdomain.com` - Stop following a domain\n" +
			"Eg: `/untrack telegram.org`\n\n" +
			"`/list` - Get a list of your followed domains\n" +
			"Eg: `/list`\n\n"
	case "list":
		output = "Here are your tracked domains:\n\n"
	default:
		output = "Didn't really get you."
	}

	return output
}

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
	return "", ""
}
