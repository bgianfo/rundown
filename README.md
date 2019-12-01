# run-down [![Build Status][travis-ci-img]][travis-ci] [![GitHub license][license-img][] [![Dependabot Status][dependabot-img]][dependabot]

An implementation of run-down protection in rust.

Run-down protection is useful when you are attempting to re-initialize or destroy a shared resource.

The pattern has two parts, a means to guarantee the resource is accessible and remains so for
the during of it's usage. As well as way to make the resource inaccessible from a point going forward
and the ability to wait for all outstanding usages to drain so you can safely perform the required operation. 

This crate was inspired by the [run-down protection primitive available in the NT Kernel][nt-run-down-docs]. 

## Usage example


````rust
use rundown::{RundownGuard, RundownRef};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn example() {

    let rundown = Arc::new(RundownRef::new());

    for _ in 0..50 {
    
        let rundown_clone = Arc::clone(&rundown);

        thread::spawn(move || {
        
            // Attempt to acquire rundown protection, while the main
            // thread could be running down the object as we execute.
            // 
            match rundown_clone.try_acquire() {
                Ok(_) => {
                    println!("{}: Run-down protection acquired.", thread::current().id());
                    
                    // Stall the thread while holding rundown protection.
                    thread::sleep(Duration::from_millis(10)); 
                }
                Err(m) => {
                    println!("{}: Failed to acquire run-down protection - {}",
                        thread::current().id(),
                        m);
                },
            }
        });
    }

    println!("{}: Waiting for rundown to complete", thread::current().id());
    rundown.wait_for_rundown();
    println!("{}: Rundown complete", thread::current().id());
}
````

## TODO

 - Add a more interesting real-world example.

 - Add some benchmarks to see if there is any opportunity to optimize the implementation.

Note: This crate is not created by, affiliated with, or supported by Microsoft.




<!-- Markdown References -->
[travis-ci]: https://travis-ci.org/bgianfo/rundown
[travis-ci-img]: https://travis-ci.org/bgianfo/rundown.svg?branch=master

[license-img]: https://img.shields.io/github/license/bgianfo/rundown.svg

[dependabot]: https://dependabot.com
[dependabot-img]: https://api.dependabot.com/badges/status?host=github&repo=bgianfo/rundown

[nt-run-down-docs]: https://docs.microsoft.com/en-us/windows-hardware/drivers/kernel/run-down-protection
