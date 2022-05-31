# Game inspired by The Binding of Isaac 
(the name of the project shall change in the future)

## Authors
- Barbara Rosiak
- Marcin Pawłowski
- Tomasz Kubica
- Dawid Mędrek

## Description
The game is heavily inspired by "[The Binding of Isaac](https://store.steampowered.com/app/113200/The_Binding_of_Isaac/)". At the moment, there is only a general idea of what the game should be like. It's going to be 2D. The player starts at a specific location in the world and has to defeat enemies on their way to the final boss. The battle system is based on shooting – just like in the original game where tha player fights with their tears as a weapon. Along the way, they can gather many different kinds of items providing bonuses to their statistics: faster shooting or movement, more powerful attacks, etc. For the time being, the map will always look and be positioned the same, but over time this can change (and probably will). Enemies will have basic AI, which will make the game both more challenging and enjoyable. All transitions in the game will be "continuous".

## Functionalities
- AI of enemies (moving, attacks, etc.)
- shooting
- character development
- bonus items
- keeping track of statistics

## Dividing the projects into parts
At the moment, it's difficult to suggest any specific divison of the project, nor can we provide a more thorough description of the two parts. However, the first one will definitely focus on the basic mechanics of the game and creating abstraction necessary for the further development of the project – a proper structure of things present in the game, "generic" methods for them (like transition of objects), etc. The second part, on the other hand, will probably be aimed at improving the elements of the game that are expected to be unpredictable, e.g. the AI of enemies or generating maps randomly.

## Part 1
What did we do?
- basic mechanics of the game and abstraction necessary for the further development
- Move system (description in file move_system.rs)
- Hitbox system
- Generating map based on a file (parsing using Serde)
- Simple main menu (using system sets and scheduling using enum AppState)
- Player shooting with arrow keys (using move system, hitbox, Query, Commands, keyboard input)
- Bullets destroying other bullets (using move system, hitbox, Query, Commands)
- Player moving with WSAD keys (using move system, hitbox, Query, Commands, keyboard input)
- Enemy moving towards player (based on the calculated distance vector and normalising it)
- Scaling window while in game (while in menu wip)
- Health system (not finished, tbd)

What will we do?
- More, different enemies and levels
- Health system, dying, despawnig...
- Work on mechanics, entity interaction (what happens when when player touches the enemy or bullet)SS
- Everithing better
- Aesthetics ✨

## Libraries
- [Bevy](https://bevyengine.org/)
