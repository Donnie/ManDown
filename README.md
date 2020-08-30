# ManDown 
[![Go Report Card](https://goreportcard.com/badge/github.com/Donnie/ManDown)](https://goreportcard.com/report/github.com/Donnie/ManDown) [![Build Status](https://api.travis-ci.org/Donnie/ManDown.svg?branch=master&status=passed)](https://travis-ci.org/github/Donnie/ManDown) [![Maintainability](https://api.codeclimate.com/v1/badges/14627b2ff38511a1eed5/maintainability)](https://codeclimate.com/github/Donnie/ManDown/maintainability) [![Test Coverage](https://api.codeclimate.com/v1/badges/14627b2ff38511a1eed5/test_coverage)](https://codeclimate.com/github/Donnie/ManDown/test_coverage)

You can use this bot to track availability of a website (or keep Heroku from sleeping :stuck_out_tongue_winking_eye:). 

The bot polls your favourite website at regular intervals to check the HTTP Status Code. 
Any change on the status code, for e.g.: 200 -> 500 or 502 -> 404, would be reported to you on your Telegram.

Try it now on: https://t.me/ManDownBot

Please also free to use this code base to run your own bot.

## Dev Setup
After setting up ngrok on 8080, you need to [setWebhook](https://core.telegram.org/bots/api#setwebhook) through Telegram using the link from Ngrok

### Start Project
Add your Telegram bot token to the .env file and then
```go run .```

## Functions
### /track
1. You send a message `/track donnieashok.in`
2. The app checks for errors 
3. If it is a fine URL then it looks up the HTTP status
4. The app logs the request in the csv file with the 5 params

### polling
1. The polling mechanism triggers the app at regular intervals. The FREQ variable in the .env file is the frequency in number of seconds.
2. The app reads the db.csv and checks if the status of the website has changed.
3. If the status has changed it sends a message to you.

### /untrack
1. You send a message `/untrack donnieashok.in`
2. The app deletes the line from the csv file with your chat_id and site

### /list
1. You send a message `/list`
2. The app replies with all the domains you are currently tracking

### /clear
1. You send a message `/clear`
2. The app clears all the domains you are currently tracking

## Improvements
I will be glad if you have suggestions on improvements or bug reports. I will be happier if you would contribute code.

## Testing
To be added soon

## Questions
Feel free to raise issues when you have questions or you are stuck somewhere.
