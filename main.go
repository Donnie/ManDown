package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log"
	"os"
	"strconv"
	"time"

	"github.com/Donnie/mandown/domains/file"
	"github.com/Donnie/mandown/domains/message"
	"github.com/Donnie/mandown/domains/web"
	"github.com/gin-gonic/gin"
	tgbotapi "github.com/go-telegram-bot-api/telegram-bot-api"
	"github.com/joho/godotenv"
)

var layout = "2006-01-02 15:04:05"

func init() {
	if err := godotenv.Load(); err != nil {
		log.Print("No .env file found")
	}
}

func main() {
	teleToken, exists := os.LookupEnv("TELEGRAM_TOKEN")
	if !exists {
		fmt.Println("Add TELEGRAM_TOKEN to .env file")
		os.Exit(1)
	}
	filename, exists := os.LookupEnv("DBFILE")
	if !exists {
		fmt.Println("Add DBFILE to .env file")
		os.Exit(1)
	}

	bot, err := tgbotapi.NewBotAPI(teleToken)
	if err != nil {
		log.Panic(err)
	}
	bot.Debug, _ = strconv.ParseBool(os.Getenv("DEBUG"))

	// Store useful items on Global context
	global := Global{Bot: bot, File: filename}

	// Start Gin Server
	r := gin.Default()

	r.POST("/hook", global.handleHook)
	r.GET("/poll", global.handlePoll)
	r.GET("/health", func(c *gin.Context) {
		c.JSON(200, nil)
	})

	r.Run()
}

func (glob *Global) handleHook(c *gin.Context) {
	buf := new(bytes.Buffer)
	buf.ReadFrom(c.Request.Body)
	str := buf.String()

	var input Input

	err := json.Unmarshal([]byte(str), &input)
	if err != nil {
		log.Panic(err)
	}

	code, _ := web.CheckHealth(*input.Message.Text)
	output := message.Process(code)
	glob.sendMessage(*input.Message.Chat.ID, output, input.Message.MessageID)

	if code != 0 && code != 1 {
		tyme := time.Now()

		record := []string{
			*input.Message.Text,
			strconv.FormatInt(*input.Message.Chat.ID, 10),
			strconv.FormatInt(*input.Message.MessageID, 10),
			tyme.Format(layout),
			strconv.Itoa(code),
		}

		file.WriteLineCSV(record, glob.File)
	}

	c.JSON(200, nil)
}

func (glob *Global) handlePoll(c *gin.Context) {
	var records [][]string
	lines, _ := file.ReadCSV(glob.File)

	for _, line := range lines {
		site := line[0]
		chatID, _ := strconv.ParseInt(line[1], 10, 64)
		msgID, _ := strconv.ParseInt(line[2], 10, 64)
		status, _ := strconv.Atoi(line[4])
		tyme, _ := time.Parse(layout, line[3])

		switch stat := status; {
		case stat > 2:
			status, _ = web.CheckHealth(site)
			if status != stat {
				tyme = time.Now()
				output := message.Process(status)
				glob.sendMessage(chatID, output, &msgID)
			}

			record := []string{
				site,
				line[1],
				line[2],
				tyme.Format(layout),
				strconv.Itoa(status),
			}
			records = append(records, record)
		}
	}

	file.WriteFileCSV(records, glob.File)
	c.JSON(200, nil)
}

func (glob *Global) sendMessage(chatID int64, text string, messageID *int64) {
	msg := tgbotapi.NewMessage(chatID, text)
	if messageID != nil {
		msg.ReplyToMessageID = int(*messageID)
	}
	glob.Bot.Send(msg)
}
