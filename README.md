Add to your cargo.toml:

`
[features]
default = ["profile"]
profile = ["screeps-timing", "screeps-timing-annotate"]

[dependencies]
screeps-timing = { git = "https://github.com/Azaril/screeps-timing", optional = true }
screeps-timing-annotate = { git = "https://github.com/Azaril/screeps-timing-annotate", optional = true }
serde = "1.0"
serde_json = "1.0"
`

Minimum setup for timing a main loop tick and dumping it to console.

`
fn main_loop() {
    #[cfg(feature = "profile")]
    {
        screeps_timing::start_trace(|| screeps::game::cpu::get_used());
    }
    
    game_loop::tick();

    #[cfg(feature = "profile")]
    {
        let trace = screeps_timing::stop_trace();

        if let Some(trace_output) = serde_json::to_string(&trace).ok() {
            info!("{}", trace_output);
        }
    }   
}
`

Annotating a function:

`
#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
fn test() {

}
`

Annotating a module:

`
#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
mod game_loop {
    pub fn tick() {

    }
}
`

Annotating an entire impl:

`
struct Foo;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl Foo {
    pub fn test() {

    }
}
`

Annotating a trait:

`
struct Foo;

#[cfg_attr(feature = "profile", screeps_timing_annotate::timing)]
impl Into<u32> for Foo {
    pub fn into(&self) -> u32 {
        0
    }
}
`