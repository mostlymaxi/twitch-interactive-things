# MostlyBot

The mostlybot is the twitch bot that I use for my stream @ https://twitch.tv/mostlymaxi. My goal for this boat is to make as easy as possible to contribute to as a viewer!

so...

## Contributing
The easiest way to contribute is by adding a command to the bot! But any and all contributions to the overall improvement of the code base are encouraged <3.

Creating a command takes a few simple steps:
1. make a fork of the main branch of this repo
2. create a module (file) at ```twitch/bot/src/commands/<command_name>.rs```
3. add this module to [src/commands/mod.rs](src/commands/mod.rs)(more details at top of file)
4. create a struct for your command and implement the ```ChatCommand``` trait (see [ping.rs](twitch/bot/src/commands/ping.rs) for a simple example)
5. document. document. document. your goal is to CONVINCE me to add this command, don't be lazy on communication
