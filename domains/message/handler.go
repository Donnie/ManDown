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
		output = "We can chat by these commands:\n" +
			"/track https://yourdomain.com - Get notified when the status of the URL changes\n" +
			"Ex: /track https://telegram.org\n\n" +
			"/untrack https://yourdomain.com - Stop following an URL\n" +
			"Ex: /untrack https://telegram.org\n"
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
