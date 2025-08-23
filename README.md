# rfetch
simple but fast fetchware alternative to neofetch, fastfetch, etc

## average times (on my system):

rfetch:
```
real  0m0.004s
user  0m0.000s
sys   0m0.004s
```

fastfetch:
```
real  0m0.015s
user  0m0.005s
sys   0m0.010s
```

screenfetch:
```
real  0m1.462s
user  0m0.949s
sys   0m0.409s
```

## displays:

`username@localhost`
   
`os/distro name`
   
`cpu model`

`kernel name & version`

`terminal`

`shell`

`window manager`

## usage:

`rfetch [OPTIONS]`

OPTIONS (optional):

`--config <FILE>     path to text file containing ascii art`
    
`--spacing <N>       spaces before ASCII art (0–255, default=3)`
    
`--color <ANSI>      (e.g. 36, 1;36, 38;5;205)`
    
`-h, --help          print help`
    
`-v, --version       print version`
    
## Examples

example with arguments:

![scrn1](https://i.imgur.com/WoRYSjY.png)

example with defaults:

![scrn2](https://i.imgur.com/i3PKCmO.png)
