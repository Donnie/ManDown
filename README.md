# ManDown 
[![Rust Report Card](https://rust-reportcard.xuri.me/badge/github.com/donnie/mandown)](https://rust-reportcard.xuri.me/report/github.com/donnie/mandown)

You can use this bot to track availability of a website (or keep Heroku from sleeping :stuck_out_tongue_winking_eye:). 

The bot polls your favourite website at regular intervals to check the HTTP Status Code. 
Any change on the status code, for e.g.: 200 -> 500 or 502 -> 404, would be reported to you on your Telegram.

Try it now on: https://t.me/ManDownBot

Please also free to use this code base to run your own bot.

### Try Project
Do `cargo run`

### Build for release
`cargo build --release`

## Dev Setup
### Start Project
Add your Telegram bot token to the .env file and then

```make dev```

## Functions
### /track
1. You send a message `/track google.in`
2. The app checks for errors in the URL string
    1. If you specify the scheme it sticks to the scheme
    2. If you do not it adds both `http` and `https`
3. If it is a fine URL then it looks up the HTTP status
4. If it does not already exist in the tracked list it adds to the list

### polling
1. The polling mechanism triggers the app at regular intervals. The FREQ variable in the .env file is the frequency in number of seconds.
2. The app reads the db.csv and checks if the status of the website has changed.
3. If the status has changed it sends a message to you.

### /untrack
1. You send a message `/untrack google.in`
2. The app checks for errors in the URL string
    1. If you specify the scheme it sticks to the scheme
    2. If you do not it adds both `http` and `https`
3. The app deletes the line from the csv file with your chat_id and site

### /list
1. You send a message `/list`
2. The app replies with all the domains you are currently tracking

### /clear
1. You send a message `/clear`
2. The app clears all the domains you are currently tracking

## Improvements
I will be glad if you have suggestions on improvements or bug reports, please make issues out of them. I will be happier if you would contribute code.

## Contributing
1. Fork it
2. Clone develop: `git clone -b develop https://github.com/Donnie/ManDown`
3. Create your feature branch: `git checkout -b new-feature`
4. Make changes and add them: `git add .`
5. Commit: `git commit -m "Add some feature"`
6. Push: `git push origin new-feature`
7. Pull request

## Questions
Feel free to raise issues when you have questions or you are stuck somewhere.
