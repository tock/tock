use crate::common::cells::{MapCell, OptionalCell, TakeCell};
use core::cell::Cell;

#[derive(Debug, Clone, Copy)]
pub enum ChunkedProcessError {
    LengthInvalid,
    Busy,
}

#[derive(Debug, Clone, Copy)]
pub enum ChunkedProcessMode {
    Read,
    Write,
    ReadWrite,
}
impl Default for ChunkedProcessMode {
    fn default() -> Self {
        ChunkedProcessMode::Read
    }
}

/// Client of a [ChunkedProcess] instance
///
/// The called function depends on the [ChunkedProcessMode] used.
///
/// - `read` means, that the data in the chunk-buffer represents the chunk
///   from the source, but new (written) data is ignored.
/// - `write` means, that the data in the chunk-buffer may not be relied on.
///   However, the new state of the buffer will be written back to the source.
/// - `read_write` combines the two options. The buffer contents are copyied from
///   the source, and then later copied back. This is the slowest option.
///
/// After the operation is complete, `chunk_done` on [ChunkedProcess] needs
/// to be called with the buffer.
pub trait ChunkedProcessClient<'a, T, R, E> {
    fn read_chunk(
        &self,
        current_pos: usize,
        acc: R,
        chunk: &'a mut [T],
        length: usize,
    ) -> Result<(), (&'a mut [T], E)>;
    fn write_chunk(
        &self,
        current_pos: usize,
        acc: R,
        chunk: &'a mut [T],
        length: usize,
    ) -> Result<(), (&'a mut [T], E)>;
    fn read_write_chunk(
        &self,
        current_pos: usize,
        acc: R,
        chunk: &'a mut [T],
        length: usize,
    ) -> Result<(), (&'a mut [T], E)>;
}

/// Process a large amount of data (e.g. [AppSlice](crate::AppSlice)) in chunks
/// supporting asynchronous and callback-style programming
///
/// This struct takes something that can be mutably borrowed and
/// for every n-bytes calls a function.
/// Depending on the mode, the buffer passed to this function may
/// contain the data of the current chunk, and/or can can be overwritten
/// with new data.
/// The chunk size is determined by the temporary buffer size.
///
/// For keeping track of custom arguments, this also implements a
/// `fold`-style accumulator for callbacks.
pub struct ChunkedProcess<'buf, 'client, S, T, R, E>
where
    S: AsMut<[T]>,
    T: Clone,
{
    src: MapCell<S>,
    buffer: TakeCell<'buf, [T]>,
    client: OptionalCell<&'client ChunkedProcessClient<'buf, T, R, E>>,

    mode: Cell<ChunkedProcessMode>,
    start: Cell<usize>,
    length: Cell<usize>,
    current_progress: OptionalCell<usize>,
}

impl<'buf, 'client, S, T, R, E> ChunkedProcess<'buf, 'client, S, T, R, E>
where
    S: AsMut<[T]>,
    T: Clone,
{
    /// Construct a new [ChunkedProcress] instance
    ///
    /// The source needs to be mutably borrowable to a slice of type T.
    pub fn new(src: S, buffer: &'buf mut [T]) -> ChunkedProcess<'buf, 'client, S, T, R, E> {
        ChunkedProcess {
            src: MapCell::new(src),
            buffer: TakeCell::new(buffer),
            client: OptionalCell::empty(),

            mode: Default::default(),
            start: Cell::new(0),
            length: Cell::new(0),
            current_progress: OptionalCell::empty(),
        }
    }

    /// Return the source and temporary buffers
    ///
    /// If no operation is currently in progress, this method will consume the
    /// [ChunkedProcess] instance and return the internal buffers.
    pub fn destroy(self) -> Result<(S, &'buf mut [T]), ChunkedProcessError> {
        if self.current_progress.is_some() {
            Err(ChunkedProcessError::Busy)
        } else {
            Ok((self.src.take().unwrap(), self.buffer.take().unwrap()))
        }
    }

    pub fn set_client(&self, client: &'client ChunkedProcessClient<'buf, T, R, E>) {
        self.client.set(client);
    }

    /// Start a [ChunkedProcess] operation
    ///
    /// `start` and `length` can be used to specify a range of the input source.
    /// `init` is the initial accumulator value.
    ///
    /// The [ChunkedProcressMode] determines whether data will be read from, written
    /// to the source, or both. See [ChunkedProcessClient] for further information.
    pub fn run(
        &self,
        mode: ChunkedProcessMode,
        start: usize,
        length: usize,
        init: R,
    ) -> Result<(), ChunkedProcessError> {
        if self.current_progress.is_some() {
            return Err(ChunkedProcessError::Busy);
        }

        self.src
            .map(|src| {
                if src.as_mut().len() < start + length {
                    Err(ChunkedProcessError::LengthInvalid)
                } else {
                    Ok(())
                }
            })
            .expect("can't map chunked progress source")?;

        self.mode.set(mode);
        self.start.set(start);
        self.length.set(length);
        self.current_progress.set(start);

        self.next_chunk(init)
            .map_or_else(|| Ok(()), |_| Err(ChunkedProcessError::LengthInvalid))?;

        Ok(())
    }

    /// Try to call the client with the next chunk
    ///
    /// If all chunks have been processed already, return a `Some(_)` with types
    /// compatible to [chunk_done].
    fn next_chunk(&self, acc: R) -> Option<(ChunkedProcessMode, Result<R, E>)> {
        let current = self
            .current_progress
            .expect("next_chunk called without progress");

        if current >= self.start.get() + self.length.get() {
            // We're done, clear the current progress to allow destroying the
            // ChunkedProcess struct
            self.current_progress.clear();
            Some((self.mode.get(), Ok(acc)))
        } else {
            let buffer = self.buffer.take().expect("buffer not available");
            let mode = self.mode.get();

            // If we read or read-write, copy the src contents into the buffer
            match mode {
                ChunkedProcessMode::Read | ChunkedProcessMode::ReadWrite => {
                    self.src
                        .map(|src| {
                            buffer
                                .iter_mut()
                                .zip((src.as_mut())[current..].iter())
                                .for_each(|(d, s)| *d = s.clone());
                        })
                        .expect("chunked_progress: src couldn't be mapped");
                }
                ChunkedProcessMode::Write => (),
            };

            let mut remaining = self.start.get() + self.length.get() - current;
            if remaining >= buffer.len() {
                remaining = buffer.len();
            }

            let chunk_res = self
                .client
                .map(move |c| match mode {
                    ChunkedProcessMode::Read => c.read_chunk(current, acc, buffer, remaining),
                    ChunkedProcessMode::Write => c.write_chunk(current, acc, buffer, remaining),
                    ChunkedProcessMode::ReadWrite => {
                        c.read_write_chunk(current, acc, buffer, remaining)
                    }
                })
                .unwrap();

            match chunk_res {
                Err((buf, err)) => {
                    // If the chunk callback encountered an error, we need
                    // to re-own the buffer so that the chunked progress can be destroyed
                    self.current_progress.clear();
                    self.buffer.replace(buf);
                    Some((self.mode.get(), Err(err)))
                }
                Ok(()) => None,
            }
        }
    }

    /// A chunk from a callback has been processed
    ///
    /// This method needs be called when a chunk (provided by a callback) has been processed.
    ///
    /// In case of an error, or when all chunks have been processed, this function will return
    /// the accumulator / error and mode used.
    pub fn chunk_done(
        &self,
        buffer: &'buf mut [T],
        res: Result<R, E>,
    ) -> Option<(ChunkedProcessMode, Result<R, E>)> {
        let prev_progress = self
            .current_progress
            .expect("chunk_done called without progress");

        // If we write or read-write, copy the buffer contents into the src slice
        match self.mode.get() {
            ChunkedProcessMode::Write | ChunkedProcessMode::ReadWrite => {
                self.src
                    .map(|src| {
                        src.as_mut()[prev_progress..]
                            .iter_mut()
                            .zip(buffer.iter())
                            .for_each(|(d, s)| *d = s.clone());
                    })
                    .expect("chunked_progress: src couldn't be mapped");
            }
            ChunkedProcessMode::Read => (),
        }

        // No matter how much we've actually processed, add a whole buffer length here
        // The only time this will be smaller is at the end - where we stop anyways
        self.current_progress.set(prev_progress + buffer.len());
        self.buffer.replace(buffer);

        match res {
            Err(e) => {
                // There was an error, immediately return it to the client
                Some((self.mode.get(), Err(e)))
            }
            Ok(acc) => {
                // There has been no error, try to call the next chunk
                self.next_chunk(acc)
            }
        }
    }
}
