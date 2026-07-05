// SPDX-License-Identifier: GPL-3.0-only
#[cfg(target_os = "windows")]
pub mod windows {
    use std::{
        io::Write,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
    };

    use windows::{
        Win32::{
            Foundation::{CloseHandle, ERROR_BROKEN_PIPE, GENERIC_WRITE},
            Storage::FileSystem::{CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_MODE, OPEN_EXISTING, PIPE_ACCESS_INBOUND, ReadFile},
            System::Pipes::{ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_TYPE_BYTE},
        },
        core::PCWSTR,
    };

    use crate::cli::display::logging::warning;

    pub fn start_pipe_server(pipe_name: &str) -> impl FnOnce() {
        let running = Arc::new(AtomicBool::new(true));
        let thread_running = running.clone();

        let pipe_name = string_to_pcwstr(pipe_name);
        let pipe_name_thread = pipe_name.clone();

        std::thread::spawn(move || {
            let handle = unsafe {
                CreateNamedPipeW(
                    PCWSTR(pipe_name_thread.as_ptr()),
                    PIPE_ACCESS_INBOUND,
                    PIPE_TYPE_BYTE,
                    1,
                    1024,
                    1024,
                    50,
                    None,
                )
            };

            let mut buf = [0u8; 1024];

            while thread_running.load(Ordering::Acquire) {
                // Wait for the next client
                unsafe {
                    let _ = ConnectNamedPipe(handle, None);
                }

                // Read lines from pipe
                loop {
                    if !thread_running.load(Ordering::Acquire) {
                        break;
                    }

                    let mut read = 0;

                    match unsafe { ReadFile(handle, Some(&mut buf), Some(&mut read), None) } {
                        Ok(_) => {
                            std::io::stdout().write_all(&buf[..read as usize]).unwrap();
                            std::io::stdout().flush().unwrap();
                        },
                        Err(e) if e.code() == ERROR_BROKEN_PIPE.into() => break,
                        Err(e) => warning!("Received error {} while reading output pipe: {}", e.code(), e.message()),
                    }
                }

                unsafe {
                    let _ = DisconnectNamedPipe(handle);
                }
            }

            unsafe {
                let _ = CloseHandle(handle);
            }
            warning!("closed handle");
        });

        move || {
            running.store(false, Ordering::Release);

            unsafe {
                let _ = CreateFileW(
                    PCWSTR(pipe_name.as_ptr()),
                    GENERIC_WRITE.0,
                    FILE_SHARE_MODE(0),
                    None,
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    None,
                );
            }
        }
    }

    fn string_to_pcwstr(string: &str) -> Vec<u16> {
        string.encode_utf16().chain(Some(0)).collect()
    }
}
