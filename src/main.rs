use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use nvml_wrapper::error::NvmlError;
use nvml_wrapper::Nvml;
use std::collections::VecDeque;
use std::io::{self};
use std::thread;
use std::time::Duration;
use sysinfo::System;
use textplots::{AxisBuilder, Chart, LineStyle, Plot, Shape};


struct FixedVec {
    data: VecDeque<(f32, f32)>,
    capacity: usize,
}

impl FixedVec {
    fn new(capacity: usize) -> Self {
        let mut data: VecDeque<(f32, f32)> = VecDeque::with_capacity(capacity);
        data.push_back((0.0, 100.0));
        data.push_back((1.0, 0.0));
        for ind in 2..capacity {
            data.push_back((ind as f32, 0.0));
        }
        FixedVec { data, capacity }
    }

    fn push(&mut self, value: (f32, f32)) {
        self.data.push_back(value);
        if self.data.len() >= self.capacity {
            self.data.pop_front();
        }
        self.data[0] = (0.0, 100.0);
        self.data[1] = (1.0, 0.0);
        //fix time
        for (i, tuple) in self.data.iter_mut().enumerate() {
            tuple.0 = i as f32;
        }
    }
}


fn main() -> Result<(), NvmlError> {

    let mut sys = System::new_all();
    let nvml = Nvml::init()?;

    // 初始化
    let mut usage_map = vec![];
    let device_count = nvml.device_count()?;
    for _ in 0..device_count {
        usage_map.push(FixedVec::new(100));
    }

    // TODO check Terminal size
    // execute!(io::stdout(), SetSize(120, 100));

    loop {
        // refresh SYS usage info
        sys.refresh_all();

        // 清屏
        print!("\x1B[2J\x1B[1;1H");
        execute!(io::stdout(), Clear(ClearType::All));

        // CPU Info
        let cpu_usage = sys.global_cpu_usage();
        // Memory Info
        let total_memory = sys.total_memory(); // in kilobytes
        let used_memory = sys.used_memory();   // in kilobytes
        println!(
            "CPU Usage: {:.2}%; Memory: {} / {} MB",
            cpu_usage,
            used_memory / 1024 / 1024,
            total_memory / 1024 / 1024
        );

        // GPU Info
        for ind in 0..device_count {
            // 这里肯定可以获取到，前面正常初始化了
            let usage_history = usage_map.get_mut(ind as usize).unwrap();

            let device = nvml.device_by_index(ind)?;
            // get GPU usage info
            let name = device.name()?;
            let mem_info = device.memory_info()?;
            let utilization = device.utilization_rates()?;
            let temperature = device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)?;

            usage_history.push((0f32, utilization.gpu as f32));

            println!("GPU {}.{} Temperature: {}°C; Utilization: {}%; Memory: {} / {} MB "
                     , ind
                     , name
                     , temperature
                     , utilization.gpu
                     , mem_info.used / 1024 / 1024, mem_info.total / 1024 / 1024
            );
            Chart::new(100, 50, 0.0, 100.0)
                .lineplot(&Shape::Lines(&(0..100).map(|i| usage_history.data[i]).collect::<Vec<_>>()))
                .y_axis_style(LineStyle::Solid)
                .x_axis_style(LineStyle::Solid)
                .display();
        }
        thread::sleep(Duration::from_millis(500));
    }
}