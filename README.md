# Twitch Interactive Things (T.I.T.S)

### Twitch Bot
- has access to twitch api
- only thing that can write to twitch api

### Data Collector
- has access to twitch api
- pulls data such as chat messages, redeems, subs...
- and pushes into seperate Franz topics :D

### TITS Service
- subscribe to necessary topics
- do things
- connect to other apis

#### Example
- subscribe to chat messages
- count number of chat messages in last 10 minutes
- set frame rate of train to number of chat messages

