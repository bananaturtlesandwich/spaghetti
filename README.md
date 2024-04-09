# `spaghetti` - a function hooker for cooked unreal engine blueprints

# [guide](GUIDE.md)

# usage
you can run spaghetti normally with/without using cli
```
spaghetti --help
Usage: spaghetti [OPTIONS] [hook] [original]

Arguments:
  [hook]      path to hook-containing blueprint
  [original]  path to original blueprint

Options:
  -v <VERSION>          engine version used to create the blueprints [default: 5.1]
  -o <output path>      path to save the hooked blueprint to [default: overwrites original]
  -h, --help            Print help
```

# limitations
- the same limitations of normal asset patching applies
- this means only one modded asset can be loaded at a time
- currently no support for hooking hooks (i need to check for existing `orig_` functions)
- if there was you could unpack, merge mod blueprints and repack for compatibility
- i could add a command which merges hooks in mod paks if that's done to make that easier too

# concept
- i modded my first unity game and thought "[Monomod HookGen](https://github.com/MonoMod/MonoMod/blob/reorganize/docs/RuntimeDetour.HookGen/Usage.md) is real nifty for hooking functions and whatnot"
- on the same day i discovered [kismet-analyzer](https://github.com/trumank/kismet-analyzer)'s `merge-functions` command which merges function kismet
- that made me think "i have decent experience dealing with assets having made stove and whatnot"
- and then i thought "i could make it like hookgen where you can call the original function in your hook"
- turns out you don't need to edit kismet
- transplanting hooks using modified code from stove works
- you can redirect the funcmap to your hook and register the original under a different name
- wow you've got hookgen but for blueprint functions
- mind blown

# credits
- [truman](https://github.com/trumank) for creating [kismet-analyzer](https://github.com/trumank/kismet-analyzer) since, although implemented differently, it helped me come up with this idea
- [atenfyr](https://github.com/atenfyr) for creating the extensive [UAssetAPI](https://github.com/atenfyr/UAssetAPI) which made this project possible ❤️
- [localcc](https://github.com/localcc) for rewriting it as [unreal_asset](https://github.com/AstroTechies/unrealmodding/tree/main/unreal_asset), allowing me to program this in [rust <img src="https://raw.githubusercontent.com/Tarikul-Islam-Anik/Animated-Fluent-Emojis/master/Emojis/Food/Crab.png" width="20" />](https://www.rust-lang.org/)

