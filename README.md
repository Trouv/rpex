# `rpex` and `xrpex`
Ratio Partition EXpression.

The main goal of this tool is to expressively split a single monitor into many virtual monitors.
`xrpex` is the binary that allows you to do so (so far in just X11, not wayland or any non-linux display servers).
`rpex` is the name of the expression that defines the ratio partition, and is written in a way that it could be applied to any other n-dimensional-rectangle-partitioning situation.

## Usage
A very explicit way to split a 16:9 monitor into two of equal size:
```sh
xrpex 8+8:9
```
The `+` indicates a partition, and the `:` indicates a ratio.

Or, say you want a square monitor on the right, and the left to be the remaining, slightly rectangular space:
```sh
xrpex 7+9:9
```
However, in this situation, you might not want to do the math in your head that `16-9=7`.
`xrpex` is smart enough to solve for missing numbers, treating the existing numbers as being "in ratio" to each other, and relying on the dimensions of the available space to figure out the rest.
So, the above example is equivalent to:
```sh
xrpex +9:9
```
And actually, now that we've omitted the 7, we can even reduce the ratio and get the same result:
```sh
xrpex +1:1
```

So, the original use case of splitting a 16:9 monitor into two of equal size could also be expressed as:
```sh
xrpex +:
```

And returning to a monitor with no partitions is as simple as
```sh
xrpex :
```

You can also partition the monitor vertically. Say you want 4 quadrants of equal size:
```sh
xrpex +:+
```
