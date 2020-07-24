## Dev Setup
After setting up ngrok on 8080, you need to [setWebhook](https://core.telegram.org/bots/api#setwebhook) through Telegram using the link from Ngrok

### Start Project
Add your Telegram bot token to the .env file and then
```go run .```

## Workflow
### /track
1. You send a message `/track donnieashok.in`
2. The app logs the request in db.csv with the 5 params

### polling
1. The polling mechanism is triggered at regular intervals.
2. The app reads the db.csv and checks if the status of the website has changed.
3. If the status changes it sends a message to you.

### /untrack
1. You send a message `/untrack donnieashok.in`
2. The app deletes the line from the csv file with your chat_id and site
