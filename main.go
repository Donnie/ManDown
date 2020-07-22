package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log"
	"os"
	"strconv"

	"github.com/Donnie/mandown/domains/message"
	"github.com/Donnie/mandown/domains/web"
	"github.com/gin-gonic/gin"
	tgbotapi "github.com/go-telegram-bot-api/telegram-bot-api"
	"github.com/joho/godotenv"
)

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

	bot, err := tgbotapi.NewBotAPI(teleToken)
	if err != nil {
		log.Panic(err)
	}
	bot.Debug, _ = strconv.ParseBool(os.Getenv("DEBUG"))

	// Store useful items on Global context
	global := Global{Bot: bot}

	// Start Gin Server
	r := gin.Default()

	r.POST("/hook", global.handleHook)
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

	msg := tgbotapi.NewMessage(*input.Message.Chat.ID, output)
	msg.ReplyToMessageID = int(*input.Message.MessageID)
	glob.Bot.Send(msg)

	c.JSON(200, nil)
}
