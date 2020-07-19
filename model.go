package main

import tgbotapi "github.com/go-telegram-bot-api/telegram-bot-api"

type Global struct {
	Bot *tgbotapi.BotAPI
}

type Input struct {
	UpdateID *int64   `json:"update_id"`
	Message  *Message `json:"message"`
}

type Message struct {
	MessageID *int64  `json:"message_id"`
	From      *From   `json:"from"`
	Chat      *Chat   `json:"chat"`
	Date      *int64  `json:"date"`
	Text      *string `json:"text"`
}

type From struct {
	ID           *int64  `json:"id"`
	IsBot        *bool   `json:"is_bot"`
	FirstName    *string `json:"first_name"`
	LastName     *string `json:"last_name"`
	Username     *string `json:"username"`
	LanguageCode *string `json:"language_code"`
}

type Chat struct {
	ID        *int64  `json:"id"`
	FirstName *string `json:"first_name"`
	LastName  *string `json:"last_name"`
	Username  *string `json:"username"`
	Type      *string `json:"type"`
}
