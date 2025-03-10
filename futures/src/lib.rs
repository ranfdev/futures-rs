//! Abstractions for asynchronous programming.
//!
//! This crate provides a number of core abstractions for writing asynchronous
//! code:
//!
//! - [Futures](crate::future) are single eventual values produced by
//!   asynchronous computations. Some programming languages (e.g. JavaScript)
//!   call this concept "promise".
//! - [Streams](crate::stream) represent a series of values
//!   produced asynchronously.
//! - [Sinks](crate::sink) provide support for asynchronous writing of
//!   data.
//! - [Executors](crate::executor) are responsible for running asynchronous
//!   tasks.
//!
//! The crate also contains abstractions for [asynchronous I/O](crate::io) and
//! [cross-task communication](crate::channel).
//!
//! Underlying all of this is the *task system*, which is a form of lightweight
//! threading. Large asynchronous computations are built up using futures,
//! streams and sinks, and then spawned as independent tasks that are run to
//! completion, but *do not block* the thread running them.
//!
//! The following example describes how the task system context is built and used
//! within macros and keywords such as async and await!.
//!
//! ```rust
//! # use futures::channel::mpsc;
//! # use futures::executor; ///standard executors to provide a context for futures and streams
//! # use futures::executor::ThreadPool;
//! # use futures::StreamExt;
//! #
//! fn main() {
//!     let pool = ThreadPool::new().expect("Failed to build pool");
//!     let (tx, rx) = mpsc::unbounded::<i32>();
//!
//!     // Create a future by an async block, where async is responsible for an
//!     // implementation of Future. At this point no executor has been provided
//!     // to this future, so it will not be running.
//!     let fut_values = async {
//!         // Create another async block, again where the Future implementation
//!         // is generated by async. Since this is inside of a parent async block,
//!         // it will be provided with the executor of the parent block when the parent
//!         // block is executed.
//!         //
//!         // This executor chaining is done by Future::poll whose second argument
//!         // is a std::task::Context. This represents our executor, and the Future
//!         // implemented by this async block can be polled using the parent async
//!         // block's executor.
//!         let fut_tx_result = async move {
//!             (0..100).for_each(|v| {
//!                 tx.unbounded_send(v).expect("Failed to send");
//!             })
//!         };
//!
//!         // Use the provided thread pool to spawn the generated future
//!         // responsible for transmission
//!         pool.spawn_ok(fut_tx_result);
//!
//!         let fut_values = rx
//!             .map(|v| v * 2)
//!             .collect();
//!
//!         // Use the executor provided to this async block to wait for the
//!         // future to complete.
//!         fut_values.await
//!     };
//!
//!     // Actually execute the above future, which will invoke Future::poll and
//!     // subsequently chain appropriate Future::poll and methods needing executors
//!     // to drive all futures. Eventually fut_values will be driven to completion.
//!     let values: Vec<i32> = executor::block_on(fut_values);
//!
//!     println!("Values={:?}", values);
//! }
//! ```
//!
//! The majority of examples and code snippets in this crate assume that they are
//! inside an async block as written above.

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    single_use_lifetimes,
    unreachable_pub
)]
#![doc(test(
    no_crate_inject,
    attr(
        deny(warnings, rust_2018_idioms, single_use_lifetimes),
        allow(dead_code, unused_assignments, unused_variables)
    )
))]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(all(feature = "bilock", not(feature = "unstable")))]
compile_error!("The `bilock` feature requires the `unstable` feature as an explicit opt-in to unstable features");

#[doc(no_inline)]
pub use futures_core::future::{Future, TryFuture};
#[doc(no_inline)]
pub use futures_util::future::{FutureExt, TryFutureExt};

#[doc(no_inline)]
pub use futures_core::stream::{Stream, TryStream};
#[doc(no_inline)]
pub use futures_util::stream::{StreamExt, TryStreamExt};

#[doc(no_inline)]
pub use futures_sink::Sink;
#[doc(no_inline)]
pub use futures_util::sink::SinkExt;

#[cfg(feature = "std")]
#[doc(no_inline)]
pub use futures_io::{AsyncBufRead, AsyncRead, AsyncSeek, AsyncWrite};
#[cfg(feature = "std")]
#[doc(no_inline)]
pub use futures_util::{AsyncBufReadExt, AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

// Macro reexports
pub use futures_core::ready; // Readiness propagation
pub use futures_util::pin_mut;
#[cfg(feature = "std")]
#[cfg(feature = "async-await")]
pub use futures_util::select;
#[cfg(feature = "async-await")]
pub use futures_util::{join, pending, poll, select_biased, try_join}; // Async-await

// Module reexports
#[doc(inline)]
pub use futures_util::{future, sink, stream, task};

#[cfg(feature = "std")]
#[cfg(feature = "async-await")]
pub use futures_util::stream_select;

#[cfg(feature = "alloc")]
#[doc(inline)]
pub use futures_channel as channel;
#[cfg(feature = "alloc")]
#[doc(inline)]
pub use futures_util::lock;

#[cfg(feature = "std")]
#[doc(inline)]
pub use futures_util::io;

#[cfg(feature = "executor")]
#[cfg_attr(docsrs, doc(cfg(feature = "executor")))]
#[doc(inline)]
pub use futures_executor as executor;

#[cfg(feature = "compat")]
#[cfg_attr(docsrs, doc(cfg(feature = "compat")))]
#[doc(inline)]
pub use futures_util::compat;

pub mod prelude {
    //! A "prelude" for crates using the `futures` crate.
    //!
    //! This prelude is similar to the standard library's prelude in that you'll
    //! almost always want to import its entire contents, but unlike the
    //! standard library's prelude you'll have to do so manually:
    //!
    //! ```
    //! # #[allow(unused_imports)]
    //! use futures::prelude::*;
    //! ```
    //!
    //! The prelude may grow over time as additional items see ubiquitous use.

    pub use crate::future::{self, Future, TryFuture};
    pub use crate::sink::{self, Sink};
    pub use crate::stream::{self, Stream, TryStream};

    #[doc(no_inline)]
    pub use crate::future::{FutureExt as _, TryFutureExt as _};
    #[doc(no_inline)]
    pub use crate::sink::SinkExt as _;
    #[doc(no_inline)]
    pub use crate::stream::{StreamExt as _, TryStreamExt as _};

    #[cfg(feature = "std")]
    pub use crate::io::{AsyncBufRead, AsyncRead, AsyncSeek, AsyncWrite};

    #[cfg(feature = "std")]
    #[doc(no_inline)]
    pub use crate::io::{
        AsyncBufReadExt as _, AsyncReadExt as _, AsyncSeekExt as _, AsyncWriteExt as _,
    };
}
