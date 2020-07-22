package message

import "fmt"

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
