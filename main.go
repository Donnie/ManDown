package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"net/url"
	"os"
	"strconv"

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

	status, code, err := checkHealth(*payload.Message.Text)

	var msg tgbotapi.MessageConfig
	if err != nil {
		msg = tgbotapi.NewMessage(*payload.Message.Chat.ID, "Gimme a correct URL")
	} else {
		if status {
			msg = tgbotapi.NewMessage(*payload.Message.Chat.ID, fmt.Sprintf("It's a %d Cap'n", code))
		} else {
			if code != 1 {
				msg = tgbotapi.NewMessage(*payload.Message.Chat.ID, fmt.Sprintf("Bad news, it says %d", code))
			} else {
				msg = tgbotapi.NewMessage(*payload.Message.Chat.ID, "Stop fooling around")
			}
		}
	}

	msg.ReplyToMessageID = int(*payload.Message.MessageID)
	glob.Bot.Send(msg)

	c.JSON(200, nil)
}

func checkHealth(site string) (bool, int, error) {
	web, err := url.ParseRequestURI(site)
	if err != nil {
		return false, 0, err
	}

	resp, err := http.Get(web.String())
	if err != nil {
		return false, 1, nil
	}

	if resp.StatusCode == 200 {
		return true, resp.StatusCode, nil
	}
	return false, resp.StatusCode, nil
}
