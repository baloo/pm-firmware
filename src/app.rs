use {
    core::cell::UnsafeCell,
    pin_utils::pin_mut,
    core::task::Poll,
};

use crate::utils::io::{self, BufReader, Read, Write};

#[derive(Debug)]
pub struct Error;

pub async fn run(input: impl Read, output: impl Write) -> Result<(), Error> {
    pin_mut!(output);
    let input = BufReader::new(input, [0; 32]);
    pin_mut!(input);
    let mut buffer = [0; 64];
    loop {
        io::write_all(output.as_mut(), "\nHello, what's your name?\n> ")
            .await
            .map_err(|_| Error)?;
        io::flush(output.as_mut()).await.map_err(|_| Error)?;
        match io::read_until(input.as_mut(), b'\r', &mut buffer[..])
            .await
            .map_err(|_| Error)?
        {
            Ok(amount) => {
                if amount == 0 {
                    io::write_all(output.as_mut(), b"\n")
                        .await
                        .map_err(|_| Error)?;
                    return Ok(());
                }
                io::write_all(output.as_mut(), "Hi ")
                    .await
                    .map_err(|_| Error)?;
                io::write_all(output.as_mut(), &buffer[..(amount - 1)])
                    .await
                    .map_err(|_| Error)?;
                io::write_all(output.as_mut(), " 👋 \n\n")
                    .await
                    .map_err(|_| Error)?;
            }
            Err(_) => {
                io::write_all(
                    output.as_mut(),
                    "\nSorry, that's a bit long for me 😭\n\n",
                )
                .await
                .map_err(|_| Error)?;
            }
        }
    }
}
