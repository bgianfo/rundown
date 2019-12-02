# run-down [![Build Status][travis-ci-img]][travis-ci] [![GitHub license][license-img]][license] [![Dependabot Status][dependabot-img]][dependabot]

An implementation of run-down protection in rust.

Run-down protection is useful when to re-initialize or destroy a shared resource in a [SMP][smp-link] runtime environment.

The pattern has two parts, a means to guarantee the resource is accessible and remains so for
the during of it's usage. As well as way to make the resource inaccessible from a point going forward
and the ability to wait for all outstanding usages to drain so you can safely perform the required operation. 

This crate was inspired by the [run-down protection primitive available in the NT Kernel][nt-run-down-docs]. 


## Usage example

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
            Ok(_) => {
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
 
 - Expand the docs before publishing the crate.

 - Add some benchmarks to see if there is any opportunity to optimize the implementation.

Note: This crate is not created by, affiliated with, or supported by Microsoft.


<!-- Markdown References -->
[travis-ci]: https://travis-ci.org/bgianfo/rust-run-down
[travis-ci-img]: https://travis-ci.org/bgianfo/rust-run-down.svg?branch=master

[license]:https://github.com/bgianfo/rust-run-down/blob/master/LICENSE
[license-img]: https://img.shields.io/github/license/bgianfo/rust-run-down.svg

[dependabot]: https://dependabot.com
[dependabot-img]: https://api.dependabot.com/badges/status?host=github&repo=bgianfo/rust-run-down

[nt-run-down-docs]: https://docs.microsoft.com/en-us/windows-hardware/drivers/kernel/run-down-protection

[smp-link]: https://en.wikipedia.org/wiki/Symmetric_multiprocessing
