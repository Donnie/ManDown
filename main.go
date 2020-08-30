package main

import (
	"fmt"
	"log"
	"os"
	"time"

	"github.com/joho/godotenv"
	tb "gopkg.in/tucnak/telebot.v2"
)

func init() {
	if _, err := os.Stat(".env.local"); os.IsNotExist(err) {
		godotenv.Load(".env")
	} else {
		godotenv.Load(".env.local")
	}
	fmt.Println("Running for " + os.Getenv("ENV"))
}

func main() {
	teleToken, exists := os.LookupEnv("TELEGRAM_TOKEN")
	if !exists {
		fmt.Println("Add TELEGRAM_TOKEN to .env file")
		os.Exit(0)
	}
	filename, exists := os.LookupEnv("DBFILE")
	if !exists {
		fmt.Println("Add DBFILE to .env file")
		os.Exit(0)
	}

	bot, err := tb.NewBot(tb.Settings{
		Token:  teleToken,
		Poller: &tb.LongPoller{Timeout: 10 * time.Second},
	})
	if err != nil {
		log.Fatal(err)
		return
	}

	gl := Global{
		Bot:  bot,
		File: filename,
	}

	freq, exists := os.LookupEnv("FREQ")
	if !exists || freq == "" {
		freq = "600"
	}
	go gl.poll(freq)

	bot.Handle("/start", gl.handleHelp)
	bot.Handle("/help", gl.handleHelp)
	bot.Handle("/about", gl.handleAbout)
	bot.Handle("/list", gl.handleList)
	bot.Handle("/clear", gl.handleClear)
	bot.Handle("/track", gl.handleTrack)
	bot.Handle("/untrack", gl.handleUntrack)
	bot.Handle(tb.OnText, func(m *tb.Message) {
		gl.Bot.Send(m.Sender, "Didn't really get you. /help")
	})

	bot.Start()
}
