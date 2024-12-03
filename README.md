brokenithmrs is an another brokenithm intended to be run on Android as a native client with a server. 
Written in Rust with very poor design decisions. 

If you want to compile this, client and server are separated into different cargo projects, for client you (preferrably) should use xbuild, and for the server you can just use cargo build. 

In the client, to close the settings window you need to press any section that is highlighted red, to reopen settings window without relaunching the app, you need to hold any section for 20 seconds.

Report bugs into issues, no feature requests unless absolutely necessary.
