# ManDown 
[![Go Report Card](https://goreportcard.com/badge/github.com/Donnie/ManDown)](https://goreportcard.com/report/github.com/Donnie/ManDown) [![Build Status](https://api.travis-ci.org/Donnie/ManDown.svg?branch=master&status=passed)](https://travis-ci.org/github/Donnie/ManDown) [![Maintainability](https://api.codeclimate.com/v1/badges/14627b2ff38511a1eed5/maintainability)](https://codeclimate.com/github/Donnie/ManDown/maintainability) [![Test Coverage](https://api.codeclimate.com/v1/badges/14627b2ff38511a1eed5/test_coverage)](https://codeclimate.com/github/Donnie/ManDown/test_coverage)

You can use this bot to track availability of a website (or keep Heroku from sleeping :stuck_out_tongue_winking_eye:). 

The bot polls your favourite website at regular intervals to check the HTTP Status Code. 
Any change on the status code, for e.g.: 200 -> 500 or 502 -> 404, would be reported to you on your Telegram.

Try it now on: https://t.me/ManDownBot

Please also free to use this code base to run your own bot.

## Porting to Rust (WIP)
### Try Project
Do `cargo run db/db.csv`

### Build for release
`cargo build --release`

## Questions
Feel free to raise issues when you have questions or you are stuck somewhere.
