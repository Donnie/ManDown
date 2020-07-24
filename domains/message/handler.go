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
	case "start":
		output = "We can chat by these commands:\n\n" +
			"`/track yourdomain.com` - Get notified when the status of your domain changes\n" +
			"Ex: `/track telegram.org`\n\n" +
			"`/untrack yourdomain.com` - Stop following a domain\n" +
			"Ex: `/untrack telegram.org`\n"
	default:
		output = "Didn't really get you."
	}

	return output
}

func ExtractMotive(text string) (string, string) {
	s := strings.Fields(text)
	if strings.Contains(s[0], "/start") {
		return "start", ""
	}
	if strings.Contains(s[0], "/track") {
		return "track", s[1]
	}
	if strings.Contains(s[0], "/untrack") {
		return "untrack", s[1]
	}
	return "", ""
}
