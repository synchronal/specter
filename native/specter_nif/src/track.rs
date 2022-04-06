use crate::state::Ref;
use crate::{atoms, task};
use rustler::{Encoder, Env, ResourceArc, Term};
use std::fs::File;
use std::io::BufReader;
use tokio::time::Duration;
use webrtc::media::io::h264_reader::H264Reader;
use webrtc::media::Sample;

#[rustler::nif]
pub fn play_from_file<'a>(
    env: Env<'a>,
    resource: ResourceArc<Ref>,
    track_uuid: Term<'a>,
    path: Term<'a>,
) -> Term<'a> {
    let mut state = match resource.0.lock() {
        Err(_) => return (atoms::error(), atoms::lock_fail()).encode(env),
        Ok(guard) => guard,
    };

    let decoded_track_uuid: String = track_uuid.decode().unwrap();
    let track = state
        .get_track_local_static_sample(&decoded_track_uuid)
        .unwrap()
        .clone();

    let pid = state.pid.clone();
    let mut msg_env = rustler::env::OwnedEnv::new();
    let decoded_path: String = path.decode().unwrap();

    // this code is taken from webrtc.rs
    // https://github.com/webrtc-rs/examples/blob/5a0e2861c66a45fca93aadf9e70a5b045b26dc9e/examples/play-from-disk-h264/play-from-disk-h264.rs#L171
    task::spawn(async move {
        // Open a H264 file and start reading using our H264Reader
        let file = File::open(&decoded_path).unwrap();
        let reader = BufReader::new(file);
        let mut h264 = H264Reader::new(reader);

        log::debug!("Play video from file {}", decoded_path);

        // It is important to use a time.Ticker instead of time.Sleep because
        // * avoids accumulating skew, just calling time.Sleep didn't compensate for the time spent parsing the data
        // * works around latency issues with Sleep
        let mut ticker = tokio::time::interval(Duration::from_millis(33));
        loop {
            let nal = match h264.next_nal() {
                Ok(nal) => nal,
                Err(err) => {
                    log::debug!("All video frames parsed and sent: {:?}", err);
                    msg_env.send_and_clear(&pid, |env| {
                        (atoms::playback_finished(), &decoded_track_uuid).encode(env)
                    });
                    break;
                }
            };

            track
                .write_sample(&Sample {
                    data: nal.data.freeze(),
                    duration: Duration::from_secs(1),
                    ..Default::default()
                })
                .await
                .unwrap();

            let _ = ticker.tick().await;
        }
    });
    atoms::ok().encode(env)
}
