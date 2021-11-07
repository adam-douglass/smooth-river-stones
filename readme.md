
# Smooth River Stones

A micro game engine for choose your own adventure stories.

- Story described in text format.
- Flow control with inventory, branch visit counting, hidden variables.

## Missing features

- scroll back 
- documentation
- multiple files

## Bugs

- trailing lines with whitespace in zone

## Control commands

- `*item +item_a -item_b`
- `*next scene-name`
- `*end`
- `*reset`
- `*set var=value`

## Examples

The demo at https://adam-douglass.github.io/smooth-river-stones/ was generated from this example:

```
default:
    After a long walk, you come to a fork in the road.

fork: ??
    ? Which way should you go?
    [left | I want to walk left.]
    [right | Right is the way for me.]

right:
    Walking down the right path you see a puppy!
    *set saw_dog
    *next end

left: 
    You wander down the left path and enjoy feeding some ducks.
    *set fed_duck

end:
    You had a very nice walk.
    (saw_dog) You saw a puppy after all!
    (fed_duck) You gave those ducks peas!
    *reset
```
