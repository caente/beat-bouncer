use cpython::{PyDict, PyResult, Python};
use rodio::Sink;
use rodio::Source;
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

#[derive(Default)]
pub struct Music {
    pub numbers: Vec<f32>,
    pub sr: usize,
}

#[derive(Default)]
pub struct Beats {
    pub music: Music,
    pub timestamps: Vec<f32>,
    pub clicks: Vec<f32>,
    pub intervals: Vec<f32>,
}

pub fn find_beats(filename: &str) -> PyResult<Beats> {
    let gil = Python::acquire_gil();
    let music = load_music(gil.python(), filename)?;
    let py = gil.python();
    let locals = PyDict::new(py);
    locals.set_item(py, "madmom", py.import("madmom")?)?;
    locals.set_item(py, "np", py.import("numpy")?)?;
    locals.set_item(py, "music", &music.numbers)?;
    locals.set_item(py, "fps", 50)?;
    let proc = py.eval(
        "madmom.features.beats.DBNBeatTrackingProcessor(fps=fps)",
        None,
        Some(&locals),
    )?;
    locals.set_item(py, "proc", &proc)?;
    let act = py.eval(
        "madmom.features.beats.RNNBeatProcessor()(np.array(music))",
        None,
        Some(&locals),
    )?;
    locals.set_item(py, "act", &act)?;
    let timestamps = py
        .eval("proc(act)", None, Some(&locals))?
        .extract::<Vec<f32>>(py)?;
    let clicks = get_clicks(py, &timestamps, music.sr, music.numbers.len())?;
    let mut intervals = beats_to_intervals(&timestamps);
    intervals.reverse();
    Ok(Beats {
        music,
        timestamps,
        clicks,
        intervals,
    })
}

fn load_music(py: Python, filename: &str) -> PyResult<Music> {
    let locals = PyDict::new(py);
    locals.set_item(py, "filename", filename)?;
    locals.set_item(py, "librosa", py.import("librosa")?)?;
    let (numbers, sr) = py
        .eval("librosa.load(filename)", None, Some(&locals))?
        .extract::<(Vec<f32>, usize)>(py)?;
    Ok(Music { numbers, sr })
}

fn get_clicks(py: Python, beats: &Vec<f32>, sr: usize, len: usize) -> PyResult<Vec<f32>> {
    let locals = PyDict::new(py);
    locals.set_item(py, "librosa", py.import("librosa")?)?;
    locals.set_item(py, "beat_times", beats)?;
    locals.set_item(py, "sr", sr)?;
    locals.set_item(py, "l", len)?;
    py.eval(
        "librosa.clicks(beat_times, sr=sr, length=l)",
        None,
        Some(&locals),
    )?
    .extract::<Vec<f32>>(py)
}

pub fn beats_to_intervals(beats: &Vec<f32>) -> Vec<f32> {
    let mut intervals: Vec<f32> = vec![];
    for i in 0..(beats.len() - 1) {
        let interval = beats[i + 1] - beats[i];
        intervals.push(interval);
    }
    intervals
}

pub fn play_beats(intervals: Vec<u64>) {
    let device = rodio::default_output_device().unwrap();
    let sink = Sink::new(&device);
    let beat_file = File::open("data/beat.wav").unwrap();
    let intervals = intervals.into_iter();
    let source = rodio::Decoder::new(BufReader::new(beat_file))
        .unwrap()
        .buffered();
    let it = intervals.map(move |interval| source.clone().delay(Duration::from_millis(interval)));
    sink.append(rodio::source::from_iter(it));
}
