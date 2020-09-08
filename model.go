package main

import (
	"time"

	tb "gopkg.in/tucnak/telebot.v2"
)

// Global holds fundamental items
type Global struct {
	Bot  *tb.Bot
	File string
}

// Record struct
type Record struct {
	Site      string
	UserID    int
	MessageID int
	Time      time.Time
	Status    int
}
