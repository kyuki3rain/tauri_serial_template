#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::{env, io, str, sync::Arc, time::Duration};

use tauri::{async_runtime::RwLock, generate_handler, Manager, State};

use futures::{stream::StreamExt, SinkExt};
use tokio::time::sleep;
use tokio_util::codec::{Decoder, Encoder};

use bytes::{BufMut, BytesMut};
use tokio_serial::SerialPortBuilderExt;

#[cfg(unix)]
const DEFAULT_TTY: &str = "/dev/cu.usbmodem11301";
#[cfg(windows)]
const DEFAULT_TTY: &str = "COM1";

struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let newline = src.as_ref().iter().position(|b| *b == b'\n');
        if let Some(n) = newline {
            let line = src.split_to(n + 1);
            return match str::from_utf8(line.as_ref()) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid String")),
            };
        }
        Ok(None)
    }
}

impl Encoder<String> for LineCodec {
    type Error = io::Error;

    fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // println!("In writer {:?}", &item);
        dst.reserve(item.len() + 1);
        dst.put(item.as_bytes());
        Ok(())
    }
}

struct TxState(Arc<RwLock<Vec<String>>>);

#[tauri::command]
fn simple_command() {
    println!("I was invoked from JS!");
}

#[tauri::command]
async fn send_p(tx: State<'_, TxState>, message: String) -> Result<(), ()> {
    (*tx.0.write().await).push(message);
    Ok(())
}

#[tokio::main]
async fn main() {
    let mut args = env::args();
    let tty_path = args.nth(1).unwrap_or_else(|| DEFAULT_TTY.into());

    let messages = Arc::new(RwLock::new(vec![]));

    let context = tauri::generate_context!();
    let mut port = tokio_serial::new(tty_path, 9600)
        .open_native_async()
        .expect("");

    #[cfg(unix)]
    port.set_exclusive(false)
        .expect("Unable to set serial port exclusive to false");

    let stream = LineCodec.framed(port);
    let (mut tx, mut rx) = stream.split();

    let messages_for_tx = Arc::clone(&messages);

    tokio::spawn(async move {
        sleep(Duration::from_secs(1)).await;

        loop {
            sleep(Duration::from_millis(100)).await;
            if messages_for_tx.read().await.is_empty() {
                continue;
            }
            let messages: Vec<String> = messages_for_tx.read().await.to_vec();
            *messages_for_tx.write().await = vec![];
            for message in messages {
                let write_result = tx.send(format!("{}", message)).await;

                match write_result {
                    Ok(_) => println!("send: {}", message),
                    Err(err) => {
                        println!("{:?}", err);
                        (*messages_for_tx.write().await).push(message);
                    }
                }
            }
        }
    });

    tauri::async_runtime::set(tokio::runtime::Handle::current());

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.app_handle();
            tokio::spawn(async move {
                loop {
                    let item = rx
                        .next()
                        .await
                        .expect("Error awaiting future in RX stream.")
                        .expect("Reading stream resulted in an error");
                    print!("receive: {item}");
                    app_handle.emit_all("serial_receiver", item).unwrap();
                }
            });
            Ok(())
        })
        .menu(tauri::Menu::os_default(&context.package_info().name))
        .manage(TxState(messages))
        .invoke_handler(generate_handler![simple_command, send_p])
        .run(context)
        .expect("error while running tauri application");
}
