package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log"
	"os"
	"strconv"

	"github.com/gin-gonic/gin"
	tgbotapi "github.com/go-telegram-bot-api/telegram-bot-api"
	"github.com/joho/godotenv"
)

func init() {
	if err := godotenv.Load(); err != nil {
		log.Print("No .env file found")
		os.Exit(1)
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

	fmt.Printf("Authorized on account %s", bot.Self.UserName)

	global := Global{Bot: bot}

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

	var payload Input

	err := json.Unmarshal([]byte(str), &payload)
	if err != nil {
		log.Panic(err)
	}

	msg := tgbotapi.NewMessage(*payload.Message.Chat.ID, "Yeah I am here")
	msg.ReplyToMessageID = int(*payload.Message.MessageID)
	glob.Bot.Send(msg)

	c.JSON(200, gin.H{
		"message": str,
	})
}
