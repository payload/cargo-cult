# ggez-sandspiel

Based upon the awesome [MaxBittker/sandspiel](https://github.com/MaxBittker/sandspiel), licensed under MIT License by Max Bittker.
The original sandspiel is built for WASM, WebGL and JavaScript and features much more! Play the original on https://sandspiel.club read [a longer post on the project](https://maxbittker.com/making-sandspiel).

This fork copies from the original the cellular automaton simulating the pixels
but removes all the WASM, WebGL and JavaScript and reimplements parts of it against [ggez](https://ggez.rs) and GLSL 1.50.
