use std::process::Command;

use corrust_lib::corruptors::{
    chain::CorruptorChain, BitOp, BitwiseCorruptor, Corruptor, Input, RandCorruptor, TiltCorruptor,
};
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Corruptor",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}

struct MyApp {
    filepath: String,
    corruptpath: String,
    chain: CorruptorChain,

    rand_corruptors: Vec<RandCorruptor>,
    bitwise_corruptors: Vec<BitwiseCorruptor>,
    tilt_corruptors: Vec<TiltCorruptor>,

    corruption_order: Vec<(String, i32)>,

    max_len: usize,
    next_corruptor: String,
    app_to_run_on_corrupt: String,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            filepath: " ".to_owned(),
            corruptpath: " ".to_owned(),
            rand_corruptors: vec![],
            bitwise_corruptors: vec![],
            tilt_corruptors: vec![],

            corruption_order: vec![],
            chain: CorruptorChain {
                og_data: Input {
                    data: vec![],
                    start_offset: 0,
                    end_offset: 0,
                },
                data: Input {
                    data: vec![],
                    start_offset: 0,
                    end_offset: 0,
                },
            },
            max_len: 0,
            next_corruptor: "Rand".to_owned(),
            app_to_run_on_corrupt: "".to_owned(),
        }
    }
}

fn reload_file(app: &mut MyApp) {
    let filedata = std::fs::read(app.filepath.trim()).expect("Unable to read file");
    app.max_len = filedata.len();
    app.chain.og_data.data = filedata.clone();
    app.chain.data.data = filedata.clone();
}

fn corrupt(app: &mut MyApp) {
    app.chain.data.data = app.chain.og_data.data.clone();
    for o in app.corruption_order.clone() {
        let fallback = RandCorruptor::default();
        let corruptor: &dyn Corruptor = match o.0.as_str() {
            "Tilt" => &app.tilt_corruptors[o.1 as usize],
            "Rand" => &app.rand_corruptors[o.1 as usize],
            "Bits" => &app.bitwise_corruptors[o.1 as usize],
            _ => &fallback,
        };
        app.chain.corrupt(corruptor);
    }
    std::fs::write(app.corruptpath.clone(), app.chain.data.data.clone());
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Settings");
            ui.horizontal(|ui| {
                let label = ui.label("File to corrupt: ");
                if ui.button("Open file...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.filepath = Some(path.display().to_string()).unwrap();
                    }
                    reload_file(self);
                }
            });
            ui.horizontal(|ui| {
                let label = ui.label("Out file: ");
                if ui.button("Open file...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_file() {
                        self.corruptpath = Some(path.display().to_string()).unwrap();
                    }
                }
            });
            if ui.button("Reload file").clicked() {
                //self.end_off = filedata.len();
                reload_file(self);
            }
            ui.add(
                egui::Slider::new(
                    &mut self.chain.data.start_offset,
                    0..=self.max_len.try_into().unwrap(),
                )
                .text("start offset"),
            );
            ui.add(
                egui::Slider::new(
                    &mut self.chain.data.end_offset,
                    self.chain.data.start_offset..=self.max_len.try_into().unwrap(),
                )
                .text("end offset"),
            );
            if ui.button("Corrupt!").clicked() {
                corrupt(self);

                if !self.app_to_run_on_corrupt.is_empty() {
                    if cfg!(target_os = "windows") {
                        Command::new("cmd")
                            .args([
                                "/C",
                                format!(
                                    "{} \"{}\"",
                                    self.app_to_run_on_corrupt.as_str(),
                                    self.corruptpath.as_str()
                                )
                                .as_str(),
                            ])
                            .output()
                            .expect("failed to execute process");
                    } else {
                        Command::new("sh")
                            .arg("-c")
                            .arg(
                                format!(
                                    "{} \"{}\"",
                                    self.app_to_run_on_corrupt.as_str(),
                                    self.corruptpath.as_str()
                                )
                                .as_str(),
                            )
                            .output()
                            .expect("failed to execute process");
                    };
                }
            }
            ui.horizontal(|ui| {
                let label = ui.label("Program to run on corrupt: ");
                ui.text_edit_singleline(&mut self.app_to_run_on_corrupt)
                    .labelled_by(label.id);
            })
        });
        egui::SidePanel::right("corruptor_chain").show(ctx, |ui| {
            ui.heading("Chain");
            ui.horizontal(|ui| {
                egui::ComboBox::from_label("Corruptor to add")
                    .selected_text(self.next_corruptor.clone())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.next_corruptor, "Tilt".to_owned(), "Tilt");
                        ui.selectable_value(&mut self.next_corruptor, "Rand".to_owned(), "Rand");
                        ui.selectable_value(&mut self.next_corruptor, "Bits".to_owned(), "Bits");
                    });

                if ui.button("Add corruptor").clicked() {
                    match self.next_corruptor.as_str() {
                        "Tilt" => {
                            self.corruption_order
                                .push(("Tilt".to_owned(), self.tilt_corruptors.len() as i32));
                            self.tilt_corruptors.push(TiltCorruptor::default());
                        }
                        "Rand" => {
                            self.corruption_order
                                .push(("Rand".to_owned(), self.rand_corruptors.len() as i32));
                            self.rand_corruptors.push(RandCorruptor::default());
                        }
                        "Bits" => {
                            self.corruption_order
                                .push(("Bits".to_owned(), self.bitwise_corruptors.len() as i32));
                            self.bitwise_corruptors.push(BitwiseCorruptor::default());
                        }
                        _ => (),
                    }
                }
            });
            if !self.corruption_order.is_empty() && ui.button("Remove corruptor").clicked() {
                let o = self.corruption_order.pop().unwrap();
                match o.0.as_str() {
                    "Tilt" => {
                        self.tilt_corruptors.pop();
                    }
                    "Rand" => {
                        self.rand_corruptors.pop();
                    }
                    "Bits" => {
                        self.bitwise_corruptors.pop();
                    }
                    _ => {}
                }
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
                for o in self.corruption_order.clone() {
                    ui.push_id(format!("{}, {}", o.0, o.1), |ui| {
                        match o.0.as_str() {
                            "Tilt" => {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    ui.heading(format!("{}, {}", o.0, o.1));
                                    let mut corruptor = &mut self.tilt_corruptors[o.1 as usize];
                                    ui.horizontal(|ui| {
                                        ui.add(
                                            egui::Slider::new(&mut corruptor.intensity, 0..=100000)
                                                .text("intensity"),
                                        );
                                        ui.add(
                                            egui::Slider::new(&mut corruptor.tilt, -10..=10)
                                                .text("tilt"),
                                        );
                                    });
                                });
                            }
                            "Rand" => {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    ui.heading(format!("{}, {}", o.0, o.1));
                                    let mut corruptor = &mut self.rand_corruptors[o.1 as usize];
                                    ui.horizontal(|ui| {
                                        ui.add(
                                            egui::Slider::new(&mut corruptor.intensity, 0..=100000)
                                                .text("intensity"),
                                        );
                                    });
                                });
                            }
                            "Bits" => {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    ui.heading(format!("{}, {}", o.0, o.1));
                                    let mut corruptor = &mut self.bitwise_corruptors[o.1 as usize];
                                    ui.horizontal(|ui| {
                                        ui.add(
                                            egui::Slider::new(&mut corruptor.intensity, 0..=100000)
                                                .text("intensity"),
                                        );
                                        egui::ComboBox::from_label("Bitwise op")
                                            .selected_text(format!("{}", corruptor.op))
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut corruptor.op,
                                                    BitOp::AND,
                                                    "AND",
                                                );
                                                ui.selectable_value(
                                                    &mut corruptor.op,
                                                    BitOp::OR,
                                                    "OR",
                                                );
                                                ui.selectable_value(
                                                    &mut corruptor.op,
                                                    BitOp::NAND,
                                                    "NAND",
                                                );
                                            });
                                        ui.add(
                                            egui::Slider::new(&mut corruptor.rhs, 0..=255)
                                                .text("bitwise comparator"),
                                        );
                                    });
                                });
                            }
                            _ => (),
                        };
                    });
                }
            });
        });
    }
}
