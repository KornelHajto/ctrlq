# CtrlQ - A Somewhat Cool Keylogger Thing

Don't know, I think its a somewhat cool project. Started as a random idea to see how much I actually type while coding. Turns out I press 'a' way too much and my spacebar is having a rough time.

This is basically a gimmicky keylogger that shows you pretty colors and tells you things about your typing you probably didn't want to know.

⚠️ **Disclaimer**: Yeah, this logs keystrokes. Don't be weird with it. Only use it on your own stuff, obviously.

## What does it do?

🔥 **Shows you numbers in real-time**
- How fast you type (spoiler: probably slower than you think)
- Total key mashing count
- Which keys you abuse the most

📊 **Has a fancy terminal thing**
- Four tabs because why not
- Updates while you type (magic!)
- Colors everywhere because terminals need more RGB

🎯 **Keyboard heatmap that actually works**
- Shows which keys you hate the most (looking at you, semicolon)
- Pretty colors from "meh" to "this key is on fire"
- Actually maps to your real keyboard layout (took way too long to figure out)

💾 **Saves your shame for posterity**
- Auto-saves so you don't lose your typing crimes
- JSON because everything is JSON these days
- Remembers your stats between sessions (unfortunately)

## How to make it work

First, get Rust. If you don't have it:

```bash
# This installs Rust. You probably already have it if you're here
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then build this thing:

```bash
git clone <wherever-this-lives>
cd ctrlq
cargo build --release

# Or if you're impatient like me
cargo build
```

## Actually using it

**Step 1:** Figure out which keyboard to spy on

```bash
sudo ./target/release/ctrlq --list-devices
```

This will dump a bunch of scary looking device paths. Pick the one that looks most keyboard-ish.

**Step 2:** Start the thing

```bash
# Let it guess your keyboard (works sometimes)
sudo ./target/release/ctrlq

# Or tell it exactly which one (recommended)
sudo ./target/release/ctrlq -d /dev/input/event8

# Run it without the fancy UI if you're boring
sudo ./target/release/ctrlq --no-ui
```

**Step 3:** Type stuff and watch the magic happen

## How to navigate this mess

- **Tab**: Switch between the four tabs (revolutionary, I know)
- **q**: Quit and return to your regularly scheduled productivity
- **r**: Reset your stats (for when the shame becomes too much)

## The four tabs of enlightenment

### 📊 Overview
The main dashboard where you realize how much time you spend deleting code you just wrote.

### 🔝 Top Keys
A leaderboard of your most abused keys. Spoiler alert: it's probably 'e' or space.

### 🔥 Heatmap
The crown jewel. A visual representation of your keyboard that shows which keys are having a bad time. Red means "this key wants to quit", gray means "I forgot this key existed".

### 📈 Sessions
Boring statistics that make you feel productive. Or unproductive. Depends on the day.

## Where it stores your typing crimes

Your keylogging data gets saved to:
- **Linux**: `~/.local/share/ctrlq/keystroke_data.json`
- **Everywhere else**: `./keystroke_data.json` (probably nowhere since this only works on Linux)

## What you'll see when it starts

```
🚀 Starting CtrlQ - Developer Keylogger
📱 Device: /dev/input/event8
📊 Statistics will be saved automatically
⚠️  This tool logs keystrokes for analysis - use responsibly!

🎯 Keylogger started on device: /dev/input/event8
📊 Data will be saved to: /home/you/.local/share/ctrlq/keystroke_data.json
```

Followed by a colorful terminal UI that updates way too frequently.

## Requirements (aka why this might not work for you)

- **Linux** (because Windows input handling is a nightmare)
- **Root privileges** (because apparently reading keyboard input requires being root)
- **A terminal that supports colors** (if you're using a black and white terminal in 2025, we need to talk)
- **Patience** (for when it inevitably breaks on your specific setup)

## Technical stuff (in case you care)

Built with:
- **Rust** 🦀 (because memory safety is cool)
- **evdev** (for talking to your keyboard at a scary low level)
- **ratatui** (for making terminals not look like 1995)
- **serde** (because JSON serialization should be easy)
- **tokio** (async stuff that probably didn't need to be async)

Probably over-engineered for what it does, but that's half the fun.

## Ethical stuff (please read this)

This tool is for:
- ✅ Figuring out your own typing patterns
- ✅ Optimizing your keyboard layout (if you're into that)
- ✅ Procrastinating from actual work
- ✅ Showing off to friends who think terminals are magic

This tool is NOT for:
- ❌ Being creepy
- ❌ Spying on other people
- ❌ Capturing passwords (seriously, don't)
- ❌ Any form of malicious activity

## Contributing (if you want to)

Got ideas? Cool. This started as a weekend project and somehow became... this. Feel free to:
- Make the heatmap even prettier
- Add more statistics that nobody asked for
- Fix the inevitable bugs I haven't found yet
- Add export features so you can share your typing shame

## License

MIT License - Do whatever you want with it. Just don't blame me when it breaks.

---

**Final warning**: This needs root to work. That's scary. You should probably read the code first to make sure I'm not doing anything sketchy. (Spoiler: I'm not, but trust nobody, including me.)

*Built during a weekend coding binge. Quality not guaranteed. Side effects may include: increased awareness of typing habits, mild obsession with keyboard statistics, and the sudden urge to optimize everything.*
