# TwitVault

![TwitVault Logo](media/logo.svg)

## Easily Archive and Search Your Twitter Data with our Syncable Desktop App

<img src="website/media/macos.jpg" width="450" />

TwitVault is a cross platform desktop app that uses the Twitter api to download
your profile data. It can also sync your profile to retrieve new data (with some limitations) and
it can import some data from the official Twitter archives. You can then browse and search
your offline Twitter data from the convenience of a native (well, partially native) deskop app.

### Features

- Import Tweets from an existing Twitter archive (see below)
- Archive your tweets.
  - Optionally including respones to your tweets
  - Optionally including tweet media (videos, images)
- Archive your mentions.
  - Optionally including user profiles
  - Optionally including profile images
- Follows and Followers
  - Optionally including user profiles
  - Optionally including profile images
- Your lists (the ones you created)
  - Optionally including member user profiles
  - Optionally including member profile images
- Search within your downloaded data [see screenshot](media/search.jpg)
- See your Tweets reverse chronological beginning with your first Tweets.
- Sync, to download newer Tweets, mentions or responses
- Runs on macOS, Linux and Windows. Can also run in the terminal.

### Limitations

Known Issue: The Windows version curently doesn't display media / images. They're being downloaded though.

There ~might~ will be bugs for various usecases that I haven't run into during my testing.

Due to API limitations, not all data can be archived. For every category, Twitter only returns a certain amount of data:

- Your Tweets: max 3.200
- Your Mentions: max 800
- Follows / Followers: No idea, but at least 5000, probably more
- Lists: Max 1000, max 5000 members per list

> Note: I can't change the description of my API Key, so don't be confused that the app describes
> itself as
> **SwiftWatch** when you're authenticating with it.

### Download / Installation

You can find a download in the release section. Or you can compile it yourself as follows:

``` sh
cargo build --release
```

Note that if you're on Linux, some dependencies need to be met. Check out the [deploy.yml](.github/workflows/deploy.yml)

> You will also need a valid Twitter API Key. Which has to be set in your shell environment.

### Twitter Archive Sync

If you already downloaded an existing Twitter Archive, you can use it to fill up any missing Tweets in your TwitVault import.

First, perform a normal TwitVault backup.

Once the backup is done, exit TwitVault, head to the Terminal, and execute the following command:

``` sh
twitvault import -c ~/twitter-archive-folder

# or on macOS
cd /Applications/TwitVault.app/Contents/MacOS/
./TwitVault import -c ~/Path/To/twitter-archive-folder
/Applications/TwitVault.app/Contents/MacOS/TwitVault import -c ~/Path/To/twitter-archive-folder
```

Afterwards, you can start TwitVault again and it will contain the Tweets.

### More Screenshots

Search:

![Search Screenshot](website/media/search.jpg)

Linux:

![Linux Screenshot](website/media/linux.jpg)

Windows:

![Windows Screenshot](website/media/windows.jpg)

Terminal:

![Terminal Screenshot](website/media/terminal.jpg)

### Where is my data stored?

The location of your data depends on your operating system:

- Linux: `/home/username/.config/twitvault`
- Windows: `C:\Users\Username\AppData\Roaming\StyleMac\TwitVault\config`
- macOS: `/Users/username/Application Support/com.StyleMac.TwitVault`

Testing these kinds of things under three different operating systems is kinda hard. So there might be bugs.
