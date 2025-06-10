# ManDown 
[![Rust Report Card](https://rust-reportcard.xuri.me/badge/github.com/donnie/mandown)](https://rust-reportcard.xuri.me/report/github.com/donnie/mandown)

You can use this bot to track availability of a website (or keep Heroku from sleeping :stuck_out_tongue_winking_eye:). 

The bot polls your favourite website at regular intervals to check the HTTP Status Code. 
Any change on the status code, for e.g.: 200 -> 500 or 502 -> 404, would be reported to you on your Telegram.

## Production use

Try it now on: https://t.me/ManDownBot

The bot lives in a Google Compute VM, and the state is stored on Google Firestore.

Infra Code is in [here](./infra/).

I have been running this bot for more than five years now. The cost is completely covered by Google Free Tier, so it can continue to stay available.

For any kind of uptime guarantees it is best to host it on your cloud account.

## Dev Setup
### Start Project
Add your Telegram bot token to the .env file and then

```bash
cargo run .
```

### Build for release
```bash
cargo build --release
```

## Functions
### /track
1. You send a message `/track google.in`
2. The app checks for errors in the URL string
3. If it is a fine URL then it looks up the HTTP status. It checks both `http` and `https`.
4. If it does not already exist in the tracked list it adds to the list

### polling
1. The polling mechanism triggers the app at regular intervals. The FREQ variable in the .env file is the frequency in number of seconds.
2. The app checks if the status of the website has changed.
3. If the status has changed it sends a message to you.

### /untrack
1. You send a message `/untrack google.in`
2. The app checks for errors in the URL string
3. The app deletes both `http` and `https` forms.

### /list
1. You send a message `/list`
2. The app replies with all the domains you are currently tracking with their status codes

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

## CI CD and Infra
1. Once your PR is submitted, the CI would automatically do some basic `cargo` checks.
2. It would run all the tests
3. Once the PR is approved and merged, pushing a new tag will trigger the build and release.

## Questions
Feel free to raise issues when you have questions or you are stuck somewhere.
