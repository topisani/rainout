use crate::error::{
    ChangeAudioBufferSizeError, ChangeAudioPortConfigError, FatalStreamError, RunConfigError,
    StreamError,
};
use crate::error_behavior::ErrorBehavior;
use crate::{platform, AudioBufferSizeConfig, Config};

/// Get the estimated total latency of a particular configuration before running it.
///
/// `None` will be returned if the latency is not known at this time.
pub fn estimated_latency(config: &Config) -> Option<u32> {
    platform::estimated_latency(config)
}

/// Get the sample rate of a particular configuration before running it.
///
/// `None` will be returned if the sample rate is not known at this time.
pub fn sample_rate(config: &Config) -> Option<u32> {
    platform::sample_rate(config)
}

/// A processor for a stream.
pub trait ProcessHandler: 'static + Send {
    /// Initialize/allocate any buffers here. This will only be called once on
    /// creation.
    fn init(&mut self, stream_info: &StreamInfo);

    /// This gets called if the user made a change to the configuration that does not
    /// require restarting the audio thread.
    fn stream_changed(&mut self, stream_info: &StreamInfo);

    /// Process the current buffers. This will always be called on a realtime thread.
    fn process(&mut self, proc_info: ProcessInfo);
}

/// An error handler for a stream.
pub trait ErrorHandler: 'static + Send + Sync {
    /// Called when a non-fatal error occurs (any error that does not require the audio
    /// thread to restart).
    fn nonfatal_error(&mut self, error: StreamError);

    /// Called when a fatal error occurs (any error that requires the audio thread to
    /// restart).
    fn fatal_error(self, error: FatalStreamError);
}

#[derive(Debug, Clone)]
pub struct StreamInfo {
    // TODO
}

pub struct ProcessInfo {
    // TODO
}

/// Run the given configuration in an audio thread.
///
/// * `config`: The configuration to use.
/// * `use_application_name`: If `Some`, then the backend will use this name as the
/// client name that appears in the audio server. This is only relevent for some
/// backends like Jack.
/// * `error_behavior`: How the system should respond to various errors.
/// * `process_handler`: An instance of your process handler.
/// * `error_handler`: An instance of your error handler.
///
/// If an error is returned, then it means the config failed to run and no audio
/// thread was spawned.
pub fn run<P: ProcessHandler, E: ErrorHandler>(
    config: &Config,
    use_application_name: Option<String>,
    error_behavior: &ErrorBehavior,
    process_handler: P,
    error_handler: E,
) -> Result<StreamHandle<P, E>, RunConfigError> {
    platform::run(config, use_application_name, error_behavior, process_handler, error_handler)
}

/// The handle to a running audio/midi stream.
///
// When this gets dropped, the stream (audio thread) will automatically stop. This
/// is the intended method for stopping a stream.
pub struct StreamHandle<P: ProcessHandler, E: ErrorHandler> {
    platform_handle: platform::PlatformStreamHandle<P, E>,
}

impl<P: ProcessHandler, E: ErrorHandler> StreamHandle<P, E> {
    /// Returns the actual configuration of the running stream. This may differ
    /// from the configuration passed into the `run()` method.
    pub fn stream_info(&self) -> &StreamInfo {
        self.platform_handle.stream_info()
    }

    /// Change the audio port configuration while the audio thread is still running.
    /// Support for this will depend on the backend.
    ///
    /// If the given config is invalid, an error will be returned with no
    /// effect on the running audio thread.
    pub fn change_audio_port_config(
        &mut self,
        audio_in_ports: Option<Vec<String>>,
        audio_out_ports: Option<Vec<String>>,
    ) -> Result<(), ChangeAudioPortConfigError> {
        self.platform_handle.change_audio_port_config(audio_in_ports, audio_out_ports)
    }

    /// Change the buffer size configuration while the audio thread is still running.
    /// Support for this will depend on the backend.
    ///
    /// If the given config is invalid, an error will be returned with no
    /// effect on the running audio thread.
    pub fn change_audio_buffer_size_config(
        &mut self,
        config: AudioBufferSizeConfig,
    ) -> Result<(), ChangeAudioBufferSizeError> {
        self.platform_handle.change_audio_buffer_size_config(config)
    }

    // It may be possible to also add `change_sample_rate_config()` here, but
    // I'm not sure how useful this would actually be.

    /// Returns whether or not this backend supports changing the audio bus
    /// configuration while the audio thread is running.
    pub fn can_change_audio_port_config(&self) -> bool {
        self.platform_handle.can_change_audio_port_config()
    }

    // Returns whether or not this backend supports changing the buffer size
    // configuration while the audio thread is running.
    pub fn can_change_audio_buffer_size_config(&self) -> bool {
        self.platform_handle.can_change_audio_buffer_size_config()
    }
}