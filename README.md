## How to use

In order to use this project, you have to run the command `cargo build -r`. This will output the executable in the `target/release/` folder with the name vpkParser.

You then have to put the executable within the same folder of the VPK decompiler with the name Decompiler. Then to use you execute the file with this structure:

```
$  ./vpkParser [location of targeted vpk file] [location of vdata inside the vpk]
```


### WARNING 
If you are trying to use this and this was not made for you this implementation is wack.

Any line with `soundevent:`, `panorama:`, or `resource_name` was thrown out because it was too hard to parse stupidly.

If the parser fails on any given line it will give up and put an empty string. 

If that works for you use this for whatevs idgaf.
