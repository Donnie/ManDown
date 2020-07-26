package main

import (
	"bytes"
	"encoding/json"
	"log"
	"strconv"
	"time"

	"github.com/Donnie/mandown/domains/file"
	"github.com/Donnie/mandown/domains/message"
	"github.com/Donnie/mandown/domains/web"
	"github.com/gin-gonic/gin"
	tgbotapi "github.com/go-telegram-bot-api/telegram-bot-api"
)

func (glob *Global) handleHook(c *gin.Context) {
	buf := new(bytes.Buffer)
	buf.ReadFrom(c.Request.Body)
	str := buf.String()

	var input Input
	var output string

	err := json.Unmarshal([]byte(str), &input)
	if err != nil {
		log.Panic(err)
	}

	motive, arg := message.ExtractMotive(*input.Message.Text)

	switch motive {
	case "clear":
		output = glob.handleClear(*input.Message)
	case "list":
		output = glob.handleList(*input.Message)
	case "track":
		output = glob.handleTrack(arg, *input.Message)
	case "untrack":
		output = glob.handleUnTrack(arg, *input.Message)
	default:
		output = message.Template(motive)
	}

	glob.sendMessage(*input.Message.Chat.ID, output, input.Message.MessageID)
	c.JSON(200, nil)
}

func (glob *Global) handlePoll(c *gin.Context) {
	var records [][]string
	var sites []string

	lines, _ := file.ReadCSV(glob.File)

	for _, line := range lines {
		sites = append(sites, line[0])
	}
	results := web.CheckBulk(sites)

	for _, line := range lines {
		site := line[0]
		chatID, _ := strconv.ParseInt(line[1], 10, 64)
		msgID, _ := strconv.ParseInt(line[2], 10, 64)
		tyme, _ := time.Parse(layout, line[3])
		status, _ := strconv.Atoi(line[4])

		for _, result := range results {
			if result.Site == site {
				if result.Status != status {
					tyme = time.Now()
					output := message.Process(result.Site, result.Status, result.Misc)
					glob.sendMessage(chatID, output, &msgID)
				}
				record := []string{
					site,
					line[1],
					line[2],
					tyme.Format(layout),
					strconv.Itoa(result.Status),
				}
				records = append(records, record)
			}
		}
	}

	file.WriteFileCSV(records, glob.File)
	c.JSON(200, nil)
}

func (glob *Global) sendMessage(chatID int64, text string, messageID *int64) {
	msg := tgbotapi.NewMessage(chatID, text)
	msg.ParseMode = "Markdown"
	msg.DisableWebPagePreview = true

	if messageID != nil {
		msg.ReplyToMessageID = int(*messageID)
	}
	glob.Bot.Send(msg)
}

func (glob *Global) handleList(msg Message) string {
	var records [][]string
	var output string
	lines, _ := file.ReadCSV(glob.File)

	for _, line := range lines {
		chatID, _ := strconv.ParseInt(line[1], 10, 64)

		if chatID != *msg.Chat.ID {
			continue
		}
		records = append(records, line)
	}

	if len(records) == 0 {
		output = message.Template("emptylist")
	} else {
		output = message.Template("list")
		for num, record := range records {
			output = output + strconv.Itoa(num+1) + ". `" + record[0] + "`\n"
		}
	}
	return output
}

func (glob *Global) handleTrack(site string, msg Message) string {
	site = web.Sanitise(site)
	result := web.CheckHealth(site)
	output := message.Process(result.Site, result.Status, result.Misc)

	if result.Status != 0 && result.Status != 1 {
		lines, _ := file.ReadCSV(glob.File)
		for _, line := range lines {
			chatID, _ := strconv.ParseInt(line[1], 10, 64)
			if site == line[0] && chatID == *msg.Chat.ID {
				return output
			}
		}

		tyme := time.Now()
		record := []string{
			site,
			strconv.FormatInt(*msg.Chat.ID, 10),
			strconv.FormatInt(*msg.MessageID, 10),
			tyme.Format(layout),
			strconv.Itoa(result.Status),
		}

		file.WriteLineCSV(record, glob.File)
	}

	return output
}

func (glob *Global) handleUnTrack(site string, msg Message) string {
	site = web.Sanitise(site)
	var records [][]string
	lines, _ := file.ReadCSV(glob.File)

	for _, line := range lines {
		chatID, _ := strconv.ParseInt(line[1], 10, 64)

		if site == line[0] && chatID == *msg.Chat.ID {
			continue
		}
		records = append(records, line)
	}

	file.WriteFileCSV(records, glob.File)
	return "Removed"
}

func (glob *Global) handleClear(msg Message) string {
	var records [][]string
	lines, _ := file.ReadCSV(glob.File)

	for _, line := range lines {
		chatID, _ := strconv.ParseInt(line[1], 10, 64)

		if chatID == *msg.Chat.ID {
			continue
		}
		records = append(records, line)
	}

	file.WriteFileCSV(records, glob.File)
	return "All clear"
}
