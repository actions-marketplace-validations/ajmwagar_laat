# LAAT: Lightweight Automated Arma Toolkit

LAAT is a CLI tool for generating Arma 3 Aux mods from simpler TOML configuration files.

## Disclaimer

The LAAT Devlopment team is not resposible for any mods or content uploaded to the Steam Workshop using LAAT. It is a tool hoping to ease the Arma 3 mod development process, not make it easier to steal other people's copyrighted works.

## Why?

I ran an Arma 3 unit for close to 6 months. At our peak we had around 50 members, each of whom would, over time, get their own custom armor added to our mod. Initially it was easy to handle the demand, make a texture, convert from png > paa, and write the nessecary configuration. However, after reaching a critical mass, the requests became too much for one developer to handle. That unit ultimately failed due to this issue.

LAAT aims to fix that, by automating as much of the Arma 3 Aux Mod development process as possible, all while-adhering to insdustry-standard software engineering practices in the process.

With LAAT you can use CI/CD to automate releases and updates to your mod, you have deterministic builds, and most importantly, you have **much** less code to mantain. 

## Installation

```bash
git clone https://github.com/ajmwagar/laat
cd laat
cargo install --path .
```

## Usage

Create a new LAAT project with `laat init`

`laat init -p myproject -a Avery Wagar`

Build your Arma 3 Mod with `laat build`

Pack your Arma 3 Mod into PBOs with `laat pack`

Create a keypair with `laat keygen <name>`

Sign your PBOs with `laat pack --sign` or `laat sign`

Release to the Steam workshop with `laat release -u <steam user> -p <steam pass> -g <steam guard code>`

## Configuration

Currently you configure the bulk of LAAT via the `LAAT.toml` file.

An example file looks like this:

```toml
prefix = "17th"
name = "17th Infantry Division"
author = "Pvt. Wagar"

# Enable the following plugins
plugins = [
  "music",
  "addons",
  "kits"
]

[pack] # PBO packing settings
excludes = ["*.png"]
include_folders = []
header_extensions = []

[release]
workshop_id = 0000000 # Steam Workshop Item ID

[kits]
file = "./kits.toml"
```

A LAAT Project might look like the following:

```
.
├── addons
│  ├── CommandLink
│  ├── Core
│  ├── Customs
│  ├── Disguise
│  ├── Factions
│  ├── JetpackPatch
│  ├── Kits
│  ├── LAATImpulsePatch
│  └── Vehicles
├── assets
│  └── music
├── build
├── keys
│  ├── 17th.bikey
│  └── 17th.biprivatekey
├── kits.toml
├── LAAT.toml
└── release
   └── @17th
```

## Compiler Plugins

Plugins are what take your assets and configuration file, and turn them into valid Arma 3 Mod Addons (i.e. the things you build into PBOs)

Currently there are only a few plugins, but many more are planned. If you'd like to see a plugin or certain Arma 3 process automated, please open an issue.

### `addons`

The `addons` plugin is the simplist. It takes all of the existing Arma 3 Addons in your `addons` folder and copies them into the `build` folder.

This enables you to use LAAT as much or as little as you'd like, by mantaining support for the existing Arma 3 Mod format and tooling.

### `music`

The `music` plugin generates a `Music` addon from your `assets/music` folder. It will generate the proper `CfgMusic` and `CfgMusicClass` entries to expose the songs of your choosing in the Zeus `Play Music` action.

The `music` plugin searches your `assets/music` folder for any `.ogg` files or subfolders. It will create a `CfgMusicClass` for each of the subfolders, and a `CfgMusic` entry for each of the `.ogg` files.

For example a LAAT project with the following format:

```
assets/music
├── 80s
│  ├── Danger_Zone.ogg
│  ├── Dont_You_Want_Me.ogg
│  ├── Enjoy_The_Silence.ogg
│  ├── Mad_World.ogg
│  ├── Psycho_Killer.ogg
│  ├── Sunglasses_At_Night.ogg
│  ├── Take_On_Me.ogg
│  ├── Tom_Sawyer.ogg
└──└── True_Survivor.ogg
```

Would generate the following addon:

```
Music
├── config.cpp
└── data
   └── Music
      ├── Danger_Zone.ogg
      ├── Dont_You_Want_Me.ogg
      ├── Enjoy_The_Silence.ogg
      ├── Mad_World.ogg
      ├── Psycho_Killer.ogg
      ├── Sunglasses_At_Night.ogg
      ├── Take_On_Me.ogg
      ├── Tom_Sawyer.ogg
      └── True_Survivor.ogg
```

The `config.cpp` would contain something like:

```cpp
class CfgPatches {
  class 17th_Music {
    units[] = {};
    weapons[] = {};
    requiredAddons[] = {};
    fileName = "17th_Music.pbo";
  };
};

class CfgMusic {
  tracks[]={
    "Take_On_Me", "Mad_World", "Danger_Zone", "Enjoy_The_Silence", "Tom_Sawyer", "Psycho_Killer", "True_Survivor", "Dont_You_Want_Me", "Sunglasses_At_Night"
  };

  class Take_On_Me {
    name = "Take On Me";
    sound[] = { "17th\Music\data\Music\Take_On_Me.ogg","db+0","1.0" };
    duration = 223;
    musicClass = "17th80s";
  };

  class Mad_World {
    name = "Mad World";
    sound[] = { "17th\Music\data\Music\Mad_World.ogg","db+0","1.0" };
    duration = 227;
    musicClass = "17th80s";
  };

  class Danger_Zone {
    name = "Danger Zone";
    sound[] = { "17th\Music\data\Music\Danger_Zone.ogg","db+0","1.0" };
    duration = 225;
    musicClass = "17th80s";
  };

  class Enjoy_The_Silence {
    name = "Enjoy The Silence";
    sound[] = { "17th\Music\data\Music\Enjoy_The_Silence.ogg","db+0","1.0" };
    duration = 280;
    musicClass = "17th80s";
  };

  class Tom_Sawyer {
    name = "Tom Sawyer";
    sound[] = { "17th\Music\data\Music\Tom_Sawyer.ogg","db+0","1.0" };
    duration = 273;
    musicClass = "17th80s";
  };

  class Psycho_Killer {
    name = "Psycho Killer";
    sound[] = { "17th\Music\data\Music\Psycho_Killer.ogg","db+0","1.0" };
    duration = 312;
    musicClass = "17th80s";
  };

  class True_Survivor {
    name = "True Survivor";
    sound[] = { "17th\Music\data\Music\True_Survivor.ogg","db+0","1.0" };
    duration = 243;
    musicClass = "17th80s";
  };

  class Dont_You_Want_Me {
    name = "Dont You Want Me";
    sound[] = { "17th\Music\data\Music\Dont_You_Want_Me.ogg","db+0","1.0" };
    duration = 206;
    musicClass = "17th80s";
  };

  class Sunglasses_At_Night {
    name = "Sunglasses At Night";
    sound[] = { "17th\Music\data\Music\Sunglasses_At_Night.ogg","db+0","1.0" };
    duration = 237;
    musicClass = "17th80s";
  };

};

class CfgMusicClasses {
  class 17thmusic {
    displayName = "[17th] music";
  };
  class 17th80s {
    displayName = "[17th] 80s";
  };
};
```


### Planned Plugins

- `armor` plugin - creating armor retextures per rank, member, etc.
- `kit` plugin - for creating "Kit Boxes" and "Armor Boxes" that assign loadouts, armour, and traits to players via `addaction` SQFs.
- `core` plugin - create the basic Arma 3 Aux mod entries to create units, objects, and more, all catagorized under your unit's name.


## Roadmap & Potential Features

- Option to swap the PBO/Binarization backend. (Currently using `armake2`, but we should optionally support others)
- LuaJIT for writing custom plugins.

