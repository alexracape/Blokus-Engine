mod node;
mod simulation;

use pyo3::prelude::*;
use pyo3::types::{PyList, PyTuple};
use simulation::play_game;
use simulation::Config;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn generate_game_data(
    num_games: usize,
    id: i32,
    config: PyObject,
    inference_queue: PyObject,
    pipe: PyObject,
) -> PyResult<Vec<(Vec<(i32, i32)>, Vec<Vec<(i32, f32)>>, Vec<f32>)>> {
    Python::with_gil(|py| {
        let config: Config = config.extract::<Config>(py).unwrap();
        let i_queue = inference_queue.bind(py);
        let r_queue = pipe.bind(py);

        let mut total = Vec::new();
        for _ in 0..num_games {
            match play_game(&config, i_queue, r_queue, id) {
                Ok(data) => total.push(data),
                Err(e) => {
                    return Err(PyErr::new::<pyo3::exceptions::PyException, _>(format!(
                        "{:?}",
                        e
                    )))
                }
            };
        }
        Ok(total)
    })
}

#[pymodule]
fn blokus_self_play(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(generate_game_data, m)?)
}
