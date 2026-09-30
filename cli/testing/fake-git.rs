// Copyright 2026 The Jujutsu Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use std::time::Instant;

/// Runs real Git, pausing after the first successful invocation to let a test
/// interleave another command. The test supplies two paths outside its working
/// copy: FAKE_GIT_READY is created here, and FAKE_GIT_RESUME is created by the
/// test to release the pause. Later invocations see READY and don't pause.
fn main() {
    let ready = PathBuf::from(std::env::var_os("FAKE_GIT_READY").unwrap());
    let resume = PathBuf::from(std::env::var_os("FAKE_GIT_RESUME").unwrap());
    let status = Command::new("git")
        .args(std::env::args_os().skip(1))
        .status()
        .expect("failed to run Git");
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    match OpenOptions::new().write(true).create_new(true).open(ready) {
        Ok(_) => {}
        Err(err) if err.kind() == ErrorKind::AlreadyExists => return,
        Err(err) => panic!("failed to signal that Git finished: {err}"),
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    while !resume.try_exists().expect("failed to check resume marker") {
        assert!(Instant::now() < deadline, "timed out waiting to resume Git");
        std::thread::sleep(Duration::from_millis(10));
    }
}
