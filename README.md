run-down [![Build Status][travis-ci-img]][travis-ci] [![][crate-img]][crate] [![GitHub license][license-img]][license] [![Dependabot Status][dependabot-img]][dependabot]
=====

The `run-down` crate provides an implementation of run-down protection.

## Overview

Run-down protection as a pattern is useful in situations where re-initialization
or destruction of a shared resource is required in a [SMP][smp-link] environment.

The pattern has two parts, a means to guarantee the resource is accessible and remains so for
the during of it's usage. As well as way to make the resource inaccessible from a point going forward
and the ability to wait for all outstanding usages to drain so you can safely perform the required operation. 

This crate was inspired by the [run-down protection primitive in the NT kernel][nt-run-down-docs].
Where it's used in situations such as driver unload, where futher access to the driver
needs to be rejected and the unloading thread must wait for inflight acesss to stop before
the driver can be completely unload.

## Documentation

[https://docs.rs/run-down](https://docs.rs/run-down)

## Example

```rust
use run_down::{
    RundownGuard,
    RundownRef
};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

let rundown = Arc::new(RundownRef::new());

for i in 1..25 {

    let rundown_clone = Arc::clone(&rundown);

    thread::spawn(move || {
    
        // Attempt to acquire rundown protection, while the main
        // thread could be running down the object as we execute.
        // 
        match rundown_clone.try_acquire() {
            Ok(run_down_guard) => {
                println!("{}: Run-down protection acquired.", i);

                // Stall the thread while holding rundown protection.
                thread::sleep(Duration::from_millis(10));
            }
            Err(m) => {
                println!("{}: Failed to acquire run-down protection - {:?}", i, m);
            },
        }
    });
}

println!("0: Waiting for rundown to complete");
rundown.wait_for_rundown();
println!("0: Rundown complete");
```

## TODO

 - Add a more interesting real-world example.
 
 - Add some benchmarks to see if there is any opportunity to optimize the implementation.

Note: This crate is not created by, affiliated with, or supported by Microsoft.

<!-- Markdown References -->
[travis-ci]: https://travis-ci.org/bgianfo/rust-run-down
[travis-ci-img]: https://travis-ci.org/bgianfo/rust-run-down.svg?branch=master

[license]:https://github.com/bgianfo/rust-run-down/blob/master/LICENSE
[license-img]: https://img.shields.io/github/license/bgianfo/rust-run-down.svg

[dependabot]: https://dependabot.com
[dependabot-img]: https://api.dependabot.com/badges/status?host=github&repo=bgianfo/rust-run-down
[crate]: https://crates.io/crates/run-down
[crate-img]:http://meritbadge.herokuapp.com/run-down

[nt-run-down-docs]: https://docs.microsoft.com/en-us/windows-hardware/drivers/kernel/run-down-protection

[smp-link]: https://en.wikipedia.org/wiki/Symmetric_multiprocessing

