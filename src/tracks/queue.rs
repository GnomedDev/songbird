use crate::{
    driver::Driver,
    events::{Event, EventContext, EventHandler, TrackEvent},
    input::Input,
    tracks::{Track, TrackHandle, TrackResult},
};
use async_trait::async_trait;
use parking_lot::Mutex;
use std::{collections::VecDeque, ops::Deref, sync::Arc, time::Duration};
use tracing::{info, warn};

/// A simple queue for several audio sources, designed to
/// play in sequence.
///
/// This makes use of [`TrackEvent`]s to determine when the current
/// song or audio file has finished before playing the next entry.
///
/// One of these is automatically included via [`Driver::queue`] when
/// the `"builtin-queue"` feature is enabled.
///
/// `examples/serenity/voice_events_queue` demonstrates how a user might manage,
/// track and use this to run a song queue in many guilds in parallel.
/// This code is trivial to extend if extra functionality is needed.
///
/// # Example
///
/// ```rust,no_run
/// use songbird::{
///     driver::Driver,
///     id::GuildId,
///     input::File,
///     tracks::TrackQueue,
/// };
/// use std::collections::HashMap;
///
/// # async {
/// let guild = GuildId(0);
/// // A Call is also valid here!
/// let mut driver: Driver = Default::default();
///
/// let mut queues: HashMap<GuildId, TrackQueue> = Default::default();
///
/// let source = File::new("../audio/my-favourite-song.mp3");
///
/// // We need to ensure that this guild has a TrackQueue created for it.
/// let queue = queues.entry(guild)
///     .or_default();
///
/// // Queueing a track is this easy!
/// queue.add_source(source.into(), &mut driver);
/// # };
/// ```
///
/// [`TrackEvent`]: crate::events::TrackEvent
/// [`Driver::queue`]: crate::driver::Driver
#[derive(Clone, Debug, Default)]
pub struct TrackQueue {
    // NOTE: the choice of a parking lot mutex is quite deliberate
    inner: Arc<Mutex<TrackQueueCore>>,
}

/// Reference to a track which is known to be part of a queue.
///
/// Instances *should not* be moved from one queue to another.
#[derive(Debug)]
pub struct Queued(TrackHandle);

impl Deref for Queued {
    type Target = TrackHandle;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Queued {
    /// Clones the inner handle
    pub fn handle(&self) -> TrackHandle {
        self.0.clone()
    }
}

#[derive(Debug, Default)]
/// Inner portion of a [`TrackQueue`].
///
/// This abstracts away thread-safety from the user,
/// and offers a convenient location to store further state if required.
///
/// [`TrackQueue`]: TrackQueue
struct TrackQueueCore {
    tracks: VecDeque<Queued>,
}

struct QueueHandler {
    remote_lock: Arc<Mutex<TrackQueueCore>>,
}

#[async_trait]
impl EventHandler for QueueHandler {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        let mut inner = self.remote_lock.lock();

        // Due to possibility that users might remove, reorder,
        // or dequeue+stop tracks, we need to verify that the FIRST
        // track is the one who has ended.
        match ctx {
            EventContext::Track(ts) => {
                // This slice should have exactly one entry.
                // If the ended track has same id as the queue head, then
                // we can progress the queue.
                if inner.tracks.front()?.uuid() != ts.first()?.1.uuid() {
                    return None;
                }
            },
            _ => return None,
        }

        let _old = inner.tracks.pop_front();

        info!("Queued track ended: {:?}.", ctx);
        info!("{} tracks remain.", inner.tracks.len());

        // Keep going until we find one track which works, or we run out.
        while let Some(new) = inner.tracks.front() {
            if new.play().is_err() {
                // Discard files which cannot be used for whatever reason.
                warn!("Track in Queue couldn't be played...");
                inner.tracks.pop_front();
            } else {
                break;
            }
        }

        None
    }
}

struct SongPreloader {
    remote_lock: Arc<Mutex<TrackQueueCore>>,
}

#[async_trait]
impl EventHandler for SongPreloader {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let inner = self.remote_lock.lock();

        if let Some(track) = inner.tracks.get(1) {
            let _ = track.0.make_playable();
        }

        None
    }
}

impl TrackQueue {
    /// Create a new, empty, track queue.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(TrackQueueCore {
                tracks: VecDeque::new(),
            })),
        }
    }

    /// Adds an audio source to the queue, to be played in the channel managed by `driver`.
    pub async fn add_source(&self, input: Input, driver: &mut Driver) -> TrackHandle {
        self.add(input.into(), driver).await
    }

    /// Adds a [`Track`] object to the queue, to be played in the channel managed by `driver`.
    ///
    /// This allows additional configuration or event handlers to be added
    /// before enqueueing the audio track. [`Track`]s will be paused pre-emptively.
    pub async fn add(&self, mut track: Track, driver: &mut Driver) -> TrackHandle {
        let preload_time = Self::get_preload_time(&mut track).await;
        let handle = driver.play(track.pause());
        self.add_raw(handle, preload_time).await
    }

    pub(crate) async fn get_preload_time(track: &mut Track) -> Option<Duration> {
        let meta = match track.input {
            Input::Lazy(ref mut rec) => rec.aux_metadata().await.ok(),
            Input::Live(_, Some(ref mut rec)) => rec.aux_metadata().await.ok(),
            _ => None,
        };

        meta.and_then(|meta| meta.duration)
    }

    /// Add a raw [`TrackHandle`] to the queue.
    /// preload_time can be specified for gapless playback
    #[inline]
    pub async fn add_raw(
        &self,
        handle: TrackHandle,
        preload_time: Option<Duration>,
    ) -> TrackHandle {
        // Attempts to start loading the next track before this one ends.
        // Idea is to provide as close to gapless playback as possible,
        // while minimising memory use.
        info!("Track added to queue.");

        let remote_lock = self.inner.clone();
        let should_play = {
            let mut inner = self.inner.lock();

            let track_handle = handle.clone();

            let _ =
                track_handle.add_event(Event::Track(TrackEvent::End), QueueHandler { remote_lock });

            if let Some(time) = preload_time {
                let preload_time: Duration =
                    time.checked_sub(Duration::from_secs(5)).unwrap_or_default();
                let remote_lock = self.inner.clone();

                let _ = track_handle
                    .add_event(Event::Delayed(preload_time), SongPreloader { remote_lock });
            }

            let out = inner.tracks.is_empty();

            inner.tracks.push_back(Queued(track_handle));

            out
        };

        if should_play {
            let _ = handle.play();
        }

        handle
    }

    /// Returns a handle to the currently playing track.
    pub fn current(&self) -> Option<TrackHandle> {
        let inner = self.inner.lock();

        inner.tracks.front().map(|h| h.handle())
    }

    /// Attempts to remove a track from the specified index.
    ///
    /// The returned entry can be readded to *this* queue via [`modify_queue`].
    ///
    /// [`modify_queue`]: TrackQueue::modify_queue
    pub fn dequeue(&self, index: usize) -> Option<Queued> {
        self.modify_queue(|vq| vq.remove(index))
    }

    /// Returns the number of tracks currently in the queue.
    pub fn len(&self) -> usize {
        let inner = self.inner.lock();

        inner.tracks.len()
    }

    /// Returns whether there are no tracks currently in the queue.
    pub fn is_empty(&self) -> bool {
        let inner = self.inner.lock();

        inner.tracks.is_empty()
    }

    /// Allows modification of the inner queue (i.e., deletion, reordering).
    ///
    /// Users must be careful to `stop` removed tracks, so as to prevent
    /// resource leaks.
    pub fn modify_queue<F, O>(&self, func: F) -> O
    where
        F: FnOnce(&mut VecDeque<Queued>) -> O,
    {
        let mut inner = self.inner.lock();
        func(&mut inner.tracks)
    }

    /// Pause the track at the head of the queue.
    pub fn pause(&self) -> TrackResult<()> {
        let inner = self.inner.lock();

        if let Some(handle) = inner.tracks.front() {
            handle.pause()
        } else {
            Ok(())
        }
    }

    /// Resume the track at the head of the queue.
    pub fn resume(&self) -> TrackResult<()> {
        let inner = self.inner.lock();

        if let Some(handle) = inner.tracks.front() {
            handle.play()
        } else {
            Ok(())
        }
    }

    /// Stop the currently playing track, and clears the queue.
    pub fn stop(&self) {
        let mut inner = self.inner.lock();

        for track in inner.tracks.drain(..) {
            // Errors when removing tracks don't really make
            // a difference: an error just implies it's already gone.
            let _ = track.stop();
        }
    }

    /// Skip to the next track in the queue, if it exists.
    pub fn skip(&self) -> TrackResult<()> {
        let inner = self.inner.lock();

        inner.stop_current()
    }

    /// Returns a list of currently queued tracks.
    ///
    /// Does not allow for modification of the queue, instead returns a snapshot of the queue at the time of calling.
    ///
    /// Use [`modify_queue`] for direct modification of the queue.
    ///
    /// [`modify_queue`]: TrackQueue::modify_queue
    pub fn current_queue(&self) -> Vec<TrackHandle> {
        let inner = self.inner.lock();

        inner.tracks.iter().map(|q| q.handle()).collect()
    }
}

impl TrackQueueCore {
    /// Skip to the next track in the queue, if it exists.
    fn stop_current(&self) -> TrackResult<()> {
        if let Some(handle) = self.tracks.front() {
            handle.stop()
        } else {
            Ok(())
        }
    }
}
