//! The purpose of `StreamerWrap` struct defined here is to expose all the user-facing methods
//! of the streamer, devices, and channels contained in the `Streamer` tree
//! as a single "flattened" struct to be able to expose them in Python.

use pyo3::prelude::*;
use pyo3::exceptions::{PyValueError, PyKeyError, PyRuntimeError};

use base_streamer::channel::BaseChan;
use base_streamer::device::BaseDev;
use base_streamer::streamer::BaseStreamer;
use base_streamer::fn_lib_tools::{FnBoxF64, FnBoxBool};

use crate::channel::{AOChan, DOChan};
use crate::device::{AODev, CommonHwCfg, DODev, NIDev};
use crate::streamer::Streamer;

#[pyclass]
pub struct StreamerWrap {
    inner: Streamer
}

#[pymethods]
impl StreamerWrap {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: Streamer::new()
        }
    }

    pub fn add_ao_dev(&mut self, name: &str, samp_rate: f64) -> PyResult<()> {
        let dev = AODev::new(name, samp_rate);
        match self.inner.add_ao_dev(dev) {
            Ok(()) => Ok(()),
            Err(msg) => Err(PyValueError::new_err(msg)),
        }
    }

    pub fn add_do_dev(&mut self, name: &str, samp_rate: f64) -> PyResult<()> {
        let dev = DODev::new(name, samp_rate);
        match self.inner.add_do_dev(dev) {
            Ok(()) => Ok(()),
            Err(msg) => Err(PyValueError::new_err(msg)),
        }
    }

    // region Hardware settings
    pub fn get_starts_last(&self) -> Option<String> {
        self.inner.get_starts_last()
    }
    #[pyo3(signature = (name))]
    pub fn set_starts_last(&mut self, name: Option<String>) {
        self.inner.set_starts_last(name)
    }

    pub fn get_ref_clk_provider(&self) -> Option<(String, String)> {
        self.inner.get_ref_clk_provider()
    }
    #[pyo3(signature = (provider))]
    pub fn set_ref_clk_provider(&mut self, provider: Option<(String, String)>) {
        self.inner.set_ref_clk_provider(provider);
    }

    pub fn reset_all(&self) -> PyResult<()> {
        match self.inner.reset_all() {
            Ok(()) => Ok(()),
            Err(msg) => Err(PyRuntimeError::new_err(msg)),
        }
    }
    // endregion

    // region Compile
    fn last_instr_end_time(&self) -> f64 {
        self.inner.last_instr_end_time()
    }

    fn total_run_time(&self) -> f64 {
        self.inner.total_run_time()
    }

    #[pyo3(signature = (stop_time=None))]
    fn compile(&mut self, stop_time: Option<f64>) -> PyResult<f64> {
        match self.inner.compile(stop_time) {
            Ok(total_run_time) => Ok(total_run_time),
            Err(msg) => Err(PyValueError::new_err(msg)),
        }
    }

    fn is_fresh_compiled(&self) -> bool {
        self.inner.is_fresh_compiled()
    }

    fn clear_edit_cache(&mut self) {
        self.inner.clear_edit_cache()
    }

    #[pyo3(signature = (reset_time=None))]
    fn add_reset_instr(&mut self, reset_time: Option<f64>) -> PyResult<()> {
        match self.inner.add_reset_instr(reset_time) {
            Ok(()) => Ok(()),
            Err(msg) => Err(PyValueError::new_err(msg))
        }
    }
    // endregion

    // region Run control
    pub fn cfg_run(&mut self, bufsize_ms: f64) -> PyResult<()> {
        match self.inner.cfg_run_(bufsize_ms) {
            Ok(()) => Ok(()),
            Err(msg) => Err(PyValueError::new_err(msg)),
        }
    }

    pub fn stream_run(&mut self, calc_next: bool) -> PyResult<()> {
        match self.inner.stream_run_(calc_next) {
            Ok(()) => Ok(()),
            Err(msg) => Err(PyRuntimeError::new_err(msg)),
        }
    }

    pub fn close_run(&mut self) -> PyResult<()> {
        match self.inner.close_run_() {
            Ok(()) => Ok(()),
            Err(msg) => Err(PyRuntimeError::new_err(msg)),
        }
    }
    // endregion
}

// region Device methods
impl StreamerWrap {
    fn assert_has_dev(&self, name: &str) -> PyResult<()> {
        if self.inner.devs().contains_key(name) {
            Ok(())
        } else {
            Err(PyKeyError::new_err(format!(
                "There is no device with name {name} registered.\n\
                The following device names are registered: {:?}",
                self.inner.devs().keys()
            )))
        }
    }

    pub fn get_dev(&self, name: &str) -> PyResult<&NIDev> {
        self.assert_has_dev(name)?;
        Ok(self.inner.devs().get(name).unwrap())
    }

    pub fn get_dev_mut(&mut self, name: &str) -> PyResult<&mut NIDev> {
        self.assert_has_dev(name)?;
        Ok(self.inner.devs_mut().get_mut(name).unwrap())
    }
}

#[pymethods]
impl StreamerWrap {
    pub fn add_ao_chan(&mut self, dev_name: &str, chan_idx: usize, dflt_val: f64, rst_val: f64) -> PyResult<()> {
        let typed_dev = self.get_dev_mut(dev_name)?;

        if let NIDev::AO(dev) = typed_dev {
            let chan = AOChan::new(chan_idx, dev.samp_rate(), dflt_val, rst_val);
            match dev.add_chan_sort(chan) {
                Ok(()) => Ok(()),
                Err(msg) => Err(PyKeyError::new_err(msg)),
            }
        } else {
            Err(PyKeyError::new_err(format!("Cannot add analog output channel to non-AO device {dev_name}")))
        }
    }

    pub fn add_do_chan(&mut self, dev_name: &str, port_idx: usize, line_idx: usize, dflt_val: bool, rst_val: bool) -> PyResult<()> {
        let typed_dev = self.get_dev_mut(dev_name)?;

        if let NIDev::DO(dev) = typed_dev {
            let chan = DOChan::new(port_idx, line_idx, dev.samp_rate(), dflt_val, rst_val);
            match dev.add_chan_sort(chan) {
                Ok(()) => Ok(()),
                Err(msg) => Err(PyKeyError::new_err(msg)),
            }
        } else {
            Err(PyKeyError::new_err(format!("Cannot add digital output channel to non-DO device {dev_name}")))
        }
    }

    pub fn dev_last_instr_end_time(&self, name: &str) -> PyResult<f64> {
        let typed_dev = self.get_dev(name)?;
        Ok(match typed_dev {
            NIDev::AO(dev) => dev.last_instr_end_time(),
            NIDev::DO(dev) => dev.last_instr_end_time(),
        })
    }

    pub fn dev_clear_edit_cache(&mut self, name: &str) -> PyResult<()> {
        let typed_dev = self.get_dev_mut(name)?;
        match typed_dev {
            NIDev::AO(dev) => dev.clear_edit_cache(),
            NIDev::DO(dev) => dev.clear_edit_cache(),
        };
        Ok(())
    }

    // region Hardware settings
    pub fn dev_get_samp_rate(&self, name: &str) -> PyResult<f64> {
        let typed_dev = self.get_dev(name)?;
        let samp_rate = match typed_dev {
            NIDev::AO(dev) => dev.samp_rate(),
            NIDev::DO(dev) => dev.samp_rate(),
        };
        Ok(samp_rate)
    }

    pub fn dev_get_start_trig_in(&self, name: &str) -> PyResult<Option<String>> {
        let ni_dev = self.get_dev(name)?;
        Ok(ni_dev.hw_cfg().start_trig_in.clone())
    }

    #[pyo3(signature = (name, term))]
    pub fn dev_set_start_trig_in(&mut self, name: &str, term: Option<String>) -> PyResult<()> {
        let ni_dev = self.get_dev_mut(name)?;
        ni_dev.hw_cfg_mut().start_trig_in = term;
        Ok(())
    }

    pub fn dev_get_start_trig_out(&self, name: &str) -> PyResult<Option<String>> {
        let ni_dev = self.get_dev(name)?;
        Ok(ni_dev.hw_cfg().start_trig_out.clone())
    }

    #[pyo3(signature = (name, term))]
    pub fn dev_set_start_trig_out(&mut self, name: &str, term: Option<String>) -> PyResult<()> {
        let ni_dev = self.get_dev_mut(name)?;
        ni_dev.hw_cfg_mut().start_trig_out = term;
        Ok(())
    }

    pub fn dev_get_samp_clk_in(&self, name: &str) -> PyResult<Option<String>> {
        let ni_dev = self.get_dev(name)?;
        Ok(ni_dev.hw_cfg().samp_clk_in.clone())
    }

    #[pyo3(signature = (name, term))]
    pub fn dev_set_samp_clk_in(&mut self, name: &str, term: Option<String>) -> PyResult<()> {
        let ni_dev = self.get_dev_mut(name)?;
        ni_dev.hw_cfg_mut().samp_clk_in = term;
        Ok(())
    }

    pub fn dev_get_samp_clk_out(&self, name: &str) -> PyResult<Option<String>> {
        let ni_dev = self.get_dev(name)?;
        Ok(ni_dev.hw_cfg().samp_clk_out.clone())
    }

    #[pyo3(signature = (name, term))]
    pub fn dev_set_samp_clk_out(&mut self, name: &str, term: Option<String>) -> PyResult<()> {
        let ni_dev = self.get_dev_mut(name)?;
        ni_dev.hw_cfg_mut().samp_clk_out = term;
        Ok(())
    }

    pub fn dev_get_ref_clk_in(&self, name: &str) -> PyResult<Option<String>> {
        let ni_dev = self.get_dev(name)?;
        Ok(ni_dev.hw_cfg().ref_clk_in.clone())
    }

    #[pyo3(signature = (name, term))]
    pub fn dev_set_ref_clk_in(&mut self, name: &str, term: Option<String>) -> PyResult<()> {
        let ni_dev = self.get_dev_mut(name)?;
        ni_dev.hw_cfg_mut().ref_clk_in = term;
        Ok(())
    }

    pub fn dev_get_min_bufwrite_timeout(&self, name: &str) -> PyResult<Option<f64>> {
        let ni_dev = self.get_dev(name)?;
        Ok(ni_dev.hw_cfg().min_bufwrite_timeout.clone())
    }

    #[pyo3(signature = (name, min_timeout))]
    pub fn dev_set_min_bufwrite_timeout(&mut self, name: &str, min_timeout: Option<f64>) -> PyResult<()> {
        let ni_dev = self.get_dev_mut(name)?;
        ni_dev.hw_cfg_mut().min_bufwrite_timeout = min_timeout;
        Ok(())
    }
    // endregion
}
// endregion

// region Channel methods
impl StreamerWrap {
    fn get_ao_chan(&self, dev_name: &str, chan_idx: usize) -> PyResult<&AOChan> {
        let typed_dev = self.get_dev(dev_name)?;

        if let NIDev::AO(dev) = typed_dev {
            let chan_name = format!("ao{chan_idx}");

            if let Some(chan) = dev.chans().get(&chan_name) {
                Ok(chan)
            } else {
                Err(PyKeyError::new_err(format!(
                    "AO device {dev_name} does not have a channel {chan_name} registered"
                )))
            }
        } else {
            Err(PyKeyError::new_err(format!(
                "Device {dev_name} is not an AO device and cannot have AO channels"
            )))
        }
    }

    fn get_do_chan(&self, dev_name: &str, port: usize, line: usize) -> PyResult<&DOChan> {
        let typed_dev = self.get_dev(dev_name)?;

        if let NIDev::DO(dev) = typed_dev {
            let chan_name = format!("port{port}/line{line}");

            if let Some(chan) = dev.chans().get(&chan_name) {
                Ok(chan)
            } else {
                Err(PyKeyError::new_err(format!(
                    "DO device {dev_name} does not have a channel {chan_name} registered"
                )))
            }
        } else {
            Err(PyKeyError::new_err(format!(
                "Device {dev_name} is not a DO device and cannot have DO channels"
            )))
        }
    }

    fn get_ao_chan_mut(&mut self, dev_name: &str, chan_idx: usize) -> PyResult<&mut AOChan> {
        let typed_dev = self.get_dev_mut(dev_name)?;

        if let NIDev::AO(dev) = typed_dev {
            let chan_name = format!("ao{chan_idx}");

            if let Some(chan) = dev.chans_mut().get_mut(&chan_name) {
                Ok(chan)
            } else {
                Err(PyKeyError::new_err(format!(
                    "AO device {dev_name} does not have a channel {chan_name} registered"
                )))
            }
        } else {
            Err(PyKeyError::new_err(format!(
                "Device {dev_name} is not an AO device and cannot have AO channels"
            )))
        }
    }

    fn get_do_chan_mut(&mut self, dev_name: &str, port: usize, line: usize) -> PyResult<&mut DOChan> {
        let typed_dev = self.get_dev_mut(dev_name)?;

        if let NIDev::DO(dev) = typed_dev {
            let chan_name = format!("port{port}/line{line}");

            if let Some(chan) = dev.chans_mut().get_mut(&chan_name) {
                Ok(chan)
            } else {
                Err(PyKeyError::new_err(format!(
                    "DO device {dev_name} does not have a channel {chan_name} registered"
                )))
            }
        } else {
            Err(PyKeyError::new_err(format!(
                "Device {dev_name} is not a DO device and cannot have DO channels"
            )))
        }
    }
}

#[pymethods]
impl StreamerWrap {
    pub fn ao_chan_name(&self, dev_name: &str, chan_idx: usize) -> PyResult<String> {
        let chan = self.get_ao_chan(dev_name, chan_idx)?;
        Ok(chan.name())
    }

    pub fn do_chan_name(&self, dev_name: &str, port: usize, line: usize) -> PyResult<String> {
        let chan = self.get_do_chan(dev_name, port, line)?;
        Ok(chan.name())
    }

    pub fn ao_chan_dflt_val(&self, dev_name: &str, chan_idx: usize) -> PyResult<f64> {
        let chan = self.get_ao_chan(dev_name, chan_idx)?;
        Ok(chan.dflt_val())
    }

    pub fn do_chan_dflt_val(&self, dev_name: &str, port: usize, line: usize) -> PyResult<bool> {
        let chan = self.get_do_chan(dev_name, port, line)?;
        Ok(chan.dflt_val())
    }

    pub fn ao_chan_rst_val(&self, dev_name: &str, chan_idx: usize) -> PyResult<f64> {
        let chan = self.get_ao_chan(dev_name, chan_idx)?;
        Ok(chan.rst_val())
    }

    pub fn do_chan_rst_val(&self, dev_name: &str, port: usize, line: usize) -> PyResult<bool> {
        let chan = self.get_do_chan(dev_name, port, line)?;
        Ok(chan.rst_val())
    }

    pub fn chan_last_instr_end_time(&self, dev_name: &str, chan_name: &str) -> PyResult<f64> {
        let dev = self.get_dev(dev_name)?;
        Ok(match dev {
            NIDev::AO(dev) => {
                dev.chans()
                    .get(chan_name)
                    .expect(&format!("Channel {chan_name} not found in device {dev_name}"))
                    .last_instr_end_time()
            },
            NIDev::DO(dev) => {
                dev.chans()
                    .get(chan_name)
                    .expect(&format!("Channel {chan_name} not found in device {dev_name}"))
                    .last_instr_end_time()
            }
        })
    }

    pub fn chan_clear_edit_cache(&mut self, dev_name: &str, chan_name: &str) -> PyResult<()> {
        let dev = self.get_dev_mut(dev_name)?;
        match dev {
            NIDev::AO(dev) => {
                dev.chans_mut()
                    .get_mut(chan_name)
                    .expect(&format!("Channel {chan_name} not found in device {dev_name}"))
                    .clear_edit_cache()
            },
            NIDev::DO(dev) => {
                dev.chans_mut()
                    .get_mut(chan_name)
                    .expect(&format!("Channel {chan_name} not found in device {dev_name}"))
                    .clear_edit_cache()
            }
        };
        Ok(())
    }

    #[pyo3(signature = (dev_name, chan_idx, func, t, dur_spec))]
    pub fn ao_chan_add_instr(
        &mut self,
        dev_name: &str, chan_idx: usize,
        func: FnBoxF64, t: f64, dur_spec: Option<(f64, bool)>
    ) -> PyResult<()> {
        let chan = self.get_ao_chan_mut(dev_name, chan_idx)?;
        let res = chan.add_instr(func.inner, t, dur_spec);
        match res {
            Ok(()) => Ok(()),
            Err(msg) => Err(PyValueError::new_err(msg)),
        }
    }

    #[pyo3(signature = (dev_name, port, line, func, t, dur_spec))]
    pub fn do_chan_add_instr(
        &mut self,
        dev_name: &str, port: usize, line: usize,
        func: FnBoxBool, t: f64, dur_spec: Option<(f64, bool)>
    ) -> PyResult<()> {
        let chan = self.get_do_chan_mut(dev_name, port, line)?;
        let res = chan.add_instr(func.inner, t, dur_spec);
        match res {
            Ok(()) => Ok(()),
            Err(msg) => Err(PyValueError::new_err(msg)),
        }
    }

    #[pyo3(signature = (dev_name, chan_idx, n_samps, start_time=None, end_time=None))]
    pub fn ao_chan_calc_nsamps(
        &self,
        dev_name: &str, chan_idx: usize,
        n_samps: usize, start_time: Option<f64>, end_time: Option<f64>,
    ) -> PyResult<Vec<f64>> {
        let chan = self.get_ao_chan(dev_name, chan_idx)?;
        let res = chan.calc_nsamps(n_samps, start_time, end_time);
        match res {
            Ok(samp_vec) => Ok(samp_vec),
            Err(msg) => Err(PyValueError::new_err(msg))
        }
    }

    #[pyo3(signature = (dev_name, port, line, n_samps, start_time=None, end_time=None))]
    pub fn do_chan_calc_nsamps(
        &self,
        dev_name: &str, port: usize, line: usize,
        n_samps: usize, start_time: Option<f64>, end_time: Option<f64>
    ) -> PyResult<Vec<bool>> {
        let chan = self.get_do_chan(dev_name, port, line)?;
        let res = chan.calc_nsamps(n_samps, start_time, end_time);
        match res {
            Ok(samp_vec) => Ok(samp_vec),
            Err(msg) => Err(PyValueError::new_err(msg))
        }
    }
}
// endregion