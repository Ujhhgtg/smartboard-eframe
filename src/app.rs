use crate::state::{
    AppState, CanvasImage, CanvasObject, CanvasShape, CanvasShapeType, CanvasText, CanvasTool,
    DynamicBrushWidthMode, RenderUpdateMode, ResizeAnchor, ResizeOperation, RotationOperation,
    ThemeMode, WindowMode,
};
use crate::utils::AppUtils;
use eframe::Frame;
use eframe::egui_wgpu::wgpu::PresentMode;
use egui::{Color32, Pos2, Shape, Stroke, ViewportCommand};
use std::sync::Arc;
use std::time::Instant;

pub struct App {
    state: AppState,
    window: Option<Arc<Frame>>,
    scale_factor: f32,
}

// struct SerializableAppState {
//     canvas_objects: Vec<crate::state::CanvasObject>,
//     brush_color: Color32,
//     brush_width: f32,
//     background_color: Color32,
//     theme_mode: ThemeMode,
// }

impl Default for App {
    fn default() -> Self {
        Self {
            state: AppState::default(),
            window: None,
            scale_factor: 1.0,
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let ctx = &cc.egui_ctx;

        let mut fonts = egui::FontDefinitions::default();

        let mut font_db = fontdb::Database::new();
        font_db.load_system_fonts();

        let cjk_font_names = [
            "Noto Sans CJK SC",
            "Noto Sans CJK",
            "Microsoft YaHei",
            "微软雅黑",
        ];

        let mut font_loaded = false;

        for font_name in &cjk_font_names {
            if let Some(face_id) = font_db.query(&fontdb::Query {
                families: &[fontdb::Family::Name(font_name)],
                weight: fontdb::Weight::NORMAL,
                stretch: fontdb::Stretch::Normal,
                style: fontdb::Style::Normal,
            }) {
                if let Some(font_data) =
                    font_db.with_face_data(face_id, |data, _| Some(data.to_vec()))
                {
                    if let Some(font_bytes) = font_data {
                        fonts.font_data.insert(
                            "cjk_font".to_owned(),
                            Arc::new(egui::FontData::from_owned(font_bytes)),
                        );

                        fonts
                            .families
                            .get_mut(&egui::FontFamily::Proportional)
                            .unwrap()
                            .insert(0, "cjk_font".to_owned());

                        fonts
                            .families
                            .get_mut(&egui::FontFamily::Monospace)
                            .unwrap()
                            .insert(0, "cjk_font".to_owned());

                        font_loaded = true;
                        break;
                    }
                }
            }
        }

        if !font_loaded {
            eprintln!("Cannot find CJK font, falling back to default");
        }

        ctx.set_fonts(fonts);

        // Load previous app state (if any)
        // if let Some(storage) = cc.storage {
        //     eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        // } else {
        Default::default()
        // }
    }

    // fn apply_present_mode(&mut self) {
    //     // Note: In eframe, present mode is handled by the framework
    //     // This is a placeholder for future implementation if needed
    // }

    // fn handle_resized(&mut self, width: u32, height: u32) {
    //     // In eframe, resizing is handled automatically
    // }

    // fn update_available_video_modes(&mut self, window: &Arc<Window>) {
    //     if let Some(monitor) = window.current_monitor() {
    //         self.state.available_video_modes = monitor.video_modes().collect();

    //         if self.state.selected_video_mode_index.is_none()
    //             && !self.state.available_video_modes.is_empty()
    //         {
    //             self.state.selected_video_mode_index = Some(0);
    //         }
    //     }
    // }
}

impl eframe::App for App {
    // fn save(&mut self, storage: &mut dyn eframe::Storage) {
    //     eframe::set_value(storage, eframe::APP_KEY, self);
    // }

    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        // self.window = Some(Arc::new(frame));
        // self.scale_factor = frame.scale_factor() as f32;

        // Set window title
        // frame.set_title("smartboard");

        // Apply initial window mode
        // if let Some(window) = self.window.as_ref() {
        //     self.apply_window_mode(window);
        // }

        // // Update available video modes
        // if let Some(window) = self.window.as_ref() {
        //     self.update_available_video_modes(window);
        // }

        // Apply theme setting
        match self.state.theme_mode {
            ThemeMode::System => {
                ctx.set_visuals(egui::Visuals::default());
            }
            ThemeMode::Light => {
                ctx.set_visuals(egui::Visuals::light());
            }
            ThemeMode::Dark => {
                ctx.set_visuals(egui::Visuals::dark());
            }
        }

        // Toolbar window
        let content_rect = ctx.available_rect();
        let margin = 20.0;

        egui::Window::new("工具栏")
            .resizable(false)
            .pivot(egui::Align2::CENTER_BOTTOM)
            .default_pos([content_rect.center().x, content_rect.max.y - margin])
            .show(ctx, |ui| {
                self.render_toolbar(ui);
            });

        // Main canvas area
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_canvas(ui);
        });

        // Handle present mode changes
        // if self.state.present_mode_changed {
        //     self.apply_present_mode();
        //     self.state.present_mode_changed = false;
        // }

        // Update FPS if enabled
        if self.state.show_fps {
            _ = self.state.fps_counter.update();
        }

        match self.state.render_update_mode {
            RenderUpdateMode::Continuous => {
                ctx.request_repaint();
            }
            RenderUpdateMode::Reactive => {}
        }
    }
}

impl App {
    fn render_toolbar(&mut self, ui: &mut egui::Ui) {
        // Tool selection
        ui.horizontal(|ui| {
            ui.label("工具:");
            let old_tool = self.state.current_tool;
            if ui
                .selectable_value(&mut self.state.current_tool, CanvasTool::Select, "选择")
                .changed()
                || ui
                    .selectable_value(&mut self.state.current_tool, CanvasTool::Brush, "画笔")
                    .changed()
                || ui
                    .selectable_value(
                        &mut self.state.current_tool,
                        CanvasTool::ObjectEraser,
                        "对象橡皮擦",
                    )
                    .changed()
                || ui
                    .selectable_value(
                        &mut self.state.current_tool,
                        CanvasTool::PixelEraser,
                        "像素橡皮擦",
                    )
                    .changed()
                || ui
                    .selectable_value(&mut self.state.current_tool, CanvasTool::Insert, "插入")
                    .changed()
                || ui
                    .selectable_value(&mut self.state.current_tool, CanvasTool::Settings, "设置")
                    .changed()
            {
                if self.state.current_tool != old_tool {
                    self.state.selected_object = None;
                }
            }
        });

        ui.separator();

        // Brush related settings
        if self.state.current_tool == CanvasTool::Brush {
            ui.horizontal(|ui| {
                ui.label("颜色:");
                let old_color = self.state.brush_color;
                if ui
                    .color_edit_button_srgba(&mut self.state.brush_color)
                    .changed()
                {
                    if self.state.is_drawing {
                        for (_touch_id, active_stroke) in self.state.active_strokes.drain() {
                            if active_stroke.points.len() > 1 {
                                self.state.canvas_objects.push(CanvasObject::Stroke(
                                    crate::state::CanvasStroke {
                                        points: active_stroke.points,
                                        widths: active_stroke.widths,
                                        color: old_color,
                                        base_width: self.state.brush_width,
                                    },
                                ));
                            }
                        }
                        self.state.is_drawing = false;
                    }
                }
            });

            // Quick color buttons
            ui.horizontal(|ui| {
                ui.label("快捷颜色:");
                for color in &self.state.quick_colors {
                    let color_name = if color.r() == 255 && color.g() == 0 && color.b() == 0 {
                        "红"
                    } else if color.r() == 255 && color.g() == 255 && color.b() == 0 {
                        "黄"
                    } else if color.r() == 0 && color.g() == 255 && color.b() == 0 {
                        "绿"
                    } else if color.r() == 0 && color.g() == 0 && color.b() == 255 {
                        "蓝"
                    } else if color.r() == 0 && color.g() == 0 && color.b() == 0 {
                        "黑"
                    } else if color.r() == 255 && color.g() == 255 && color.b() == 255 {
                        "白"
                    } else {
                        "自定义"
                    };
                    if ui
                        .add(egui::Button::new(
                            egui::RichText::new(color_name).color(*color),
                        ))
                        .clicked()
                    {
                        self.state.brush_color = *color;
                    }
                }
            });

            ui.horizontal(|ui| {
                ui.label("宽度:");
                let slider_response =
                    ui.add(egui::Slider::new(&mut self.state.brush_width, 1.0..=20.0));

                if slider_response.dragged() || slider_response.hovered() {
                    self.state.show_size_preview = true;
                } else if !slider_response.dragged() && !slider_response.hovered() {
                    self.state.show_size_preview = false;
                }
            });

            // Brush width quick buttons
            ui.horizontal(|ui| {
                ui.label("快捷宽度:");
                if ui.button("小").clicked() {
                    self.state.brush_width = 1.0;
                }
                if ui.button("中").clicked() {
                    self.state.brush_width = 3.0;
                }
                if ui.button("大").clicked() {
                    self.state.brush_width = 5.0;
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("动态画笔宽度微调:");
                ui.selectable_value(
                    &mut self.state.dynamic_brush_width_mode,
                    DynamicBrushWidthMode::Disabled,
                    "禁用",
                );
                ui.selectable_value(
                    &mut self.state.dynamic_brush_width_mode,
                    DynamicBrushWidthMode::BrushTip,
                    "模拟笔锋",
                );
                ui.selectable_value(
                    &mut self.state.dynamic_brush_width_mode,
                    DynamicBrushWidthMode::SpeedBased,
                    "基于速度",
                );
            });

            ui.horizontal(|ui| {
                ui.label("笔迹平滑:");
                ui.checkbox(&mut self.state.stroke_smoothing, "启用");
            });
        }

        // Eraser related settings
        if self.state.current_tool == CanvasTool::ObjectEraser
            || self.state.current_tool == CanvasTool::PixelEraser
        {
            ui.horizontal(|ui| {
                ui.label("橡皮擦大小:");
                let slider_response =
                    ui.add(egui::Slider::new(&mut self.state.eraser_size, 5.0..=50.0));

                if slider_response.dragged() || slider_response.hovered() {
                    self.state.show_size_preview = true;
                } else if !slider_response.dragged() && !slider_response.hovered() {
                    self.state.show_size_preview = false;
                }

                if ui.button("清空画布").clicked() {
                    self.state.canvas_objects.clear();
                    self.state.active_strokes.clear();
                    self.state.is_drawing = false;
                    self.state.selected_object = None;
                    self.state.current_tool = CanvasTool::Brush;
                }
            });
        }

        // Insert tool related settings
        if self.state.current_tool == CanvasTool::Insert {
            ui.horizontal(|ui| {
                if ui.button("图片").clicked() {
                    let future = async {
                        rfd::AsyncFileDialog::new()
                            .add_filter(
                                "图片",
                                &[
                                    "png", "jpg", "jpeg", "bmp", "gif", "tiff", "pnm", "webp",
                                    "tga", "dds", "ico", "hdr", "avif", "qoi",
                                ],
                            )
                            .pick_file()
                            .await
                    };
                    // if let Some(path) = rfd::FileDialog::new()
                    //     .add_filter(
                    //         "图片",
                    //         &[
                    //             "png", "jpg", "jpeg", "bmp", "gif", "tiff", "pnm", "webp", "tga",
                    //             "dds", "ico", "hdr", "avif", "qoi",
                    //         ],
                    //     )
                    //     .pick_file()
                    // {
                    if let Some(path) = futures::executor::block_on(future) {
                        if let Ok(img) = image::open(path.path()) {
                            let img = img.to_rgba8();
                            let (width, height) = img.dimensions();
                            let aspect_ratio = width as f32 / height as f32;

                            let target_width = 300.0f32;
                            let target_height = target_width / aspect_ratio;

                            let ctx = ui.ctx();
                            let texture = ctx.load_texture(
                                "inserted_image",
                                egui::ColorImage::from_rgba_unmultiplied(
                                    [width as usize, height as usize],
                                    &img,
                                ),
                                egui::TextureOptions::LINEAR,
                            );

                            self.state
                                .canvas_objects
                                .push(CanvasObject::Image(CanvasImage {
                                    texture,
                                    pos: Pos2::new(100.0, 100.0),
                                    size: egui::vec2(target_width, target_height),
                                    aspect_ratio,
                                    marked_for_deletion: false,
                                }));
                        }
                    }
                    // }
                }
                if ui.button("文本").clicked() {
                    self.state.show_text_dialog = true;
                }
                if ui.button("形状").clicked() {
                    self.state.show_shape_dialog = true;
                }
            });

            if self.state.show_text_dialog {
                let content_rect = ui.ctx().available_rect();
                let center_pos = content_rect.center();

                egui::Window::new("插入文本")
                    .collapsible(false)
                    .resizable(false)
                    .pivot(egui::Align2::CENTER_CENTER)
                    .default_pos([center_pos.x, center_pos.y])
                    .show(ui.ctx(), |ui| {
                        ui.horizontal(|ui| {
                            ui.label("文本内容:");
                            ui.text_edit_singleline(&mut self.state.new_text_content);
                        });

                        ui.horizontal(|ui| {
                            if ui.button("确认").clicked() {
                                self.state
                                    .canvas_objects
                                    .push(CanvasObject::Text(CanvasText {
                                        text: self.state.new_text_content.clone(),
                                        pos: Pos2::new(100.0, 100.0),
                                        color: Color32::WHITE,
                                        font_size: 16.0,
                                    }));
                                self.state.show_text_dialog = false;
                                self.state.new_text_content.clear();
                            }

                            if ui.button("取消").clicked() {
                                self.state.show_text_dialog = false;
                                self.state.new_text_content.clear();
                            }
                        });
                    });
            }

            if self.state.show_shape_dialog {
                let content_rect = ui.ctx().available_rect();
                let center_pos = content_rect.center();

                egui::Window::new("插入形状")
                    .collapsible(false)
                    .resizable(false)
                    .pivot(egui::Align2::CENTER_CENTER)
                    .default_pos([center_pos.x, center_pos.y])
                    .show(ui.ctx(), |ui| {
                        ui.label("选择要插入的形状:");

                        ui.horizontal(|ui| {
                            if ui.button("线").clicked() {
                                self.state
                                    .canvas_objects
                                    .push(CanvasObject::Shape(CanvasShape {
                                        shape_type: CanvasShapeType::Line,
                                        pos: Pos2::new(100.0, 100.0),
                                        size: 100.0,
                                        color: Color32::WHITE,
                                        rotation: 0.0,
                                    }));
                                self.state.show_shape_dialog =
                                    self.state.keep_insertion_window_open;
                            }

                            if ui.button("箭头").clicked() {
                                self.state
                                    .canvas_objects
                                    .push(CanvasObject::Shape(CanvasShape {
                                        shape_type: CanvasShapeType::Arrow,
                                        pos: Pos2::new(100.0, 100.0),
                                        size: 100.0,
                                        color: Color32::WHITE,
                                        rotation: 0.0,
                                    }));
                                self.state.show_shape_dialog =
                                    self.state.keep_insertion_window_open;
                            }

                            if ui.button("矩形").clicked() {
                                self.state
                                    .canvas_objects
                                    .push(CanvasObject::Shape(CanvasShape {
                                        shape_type: CanvasShapeType::Rectangle,
                                        pos: Pos2::new(100.0, 100.0),
                                        size: 100.0,
                                        color: Color32::WHITE,
                                        rotation: 0.0,
                                    }));
                                self.state.show_shape_dialog =
                                    self.state.keep_insertion_window_open;
                            }
                            if ui.button("三角形").clicked() {
                                self.state
                                    .canvas_objects
                                    .push(CanvasObject::Shape(CanvasShape {
                                        shape_type: CanvasShapeType::Triangle,
                                        pos: Pos2::new(100.0, 100.0),
                                        size: 100.0,
                                        color: Color32::WHITE,
                                        rotation: 0.0,
                                    }));
                                self.state.show_shape_dialog =
                                    self.state.keep_insertion_window_open;
                            }

                            if ui.button("圆形").clicked() {
                                self.state
                                    .canvas_objects
                                    .push(CanvasObject::Shape(CanvasShape {
                                        shape_type: CanvasShapeType::Circle,
                                        pos: Pos2::new(100.0, 100.0),
                                        size: 100.0,
                                        color: Color32::WHITE,
                                        rotation: 0.0,
                                    }));
                                self.state.show_shape_dialog =
                                    self.state.keep_insertion_window_open;
                            }
                        });

                        ui.horizontal(|ui| {
                            if ui.button("取消").clicked() {
                                self.state.show_shape_dialog = false;
                            }
                            ui.checkbox(&mut self.state.keep_insertion_window_open, "保持窗口开启");
                        });
                    });
            }
        }

        // Settings tool related settings
        if self.state.current_tool == CanvasTool::Settings {
            ui.collapsing("外观", |ui| {
                ui.horizontal(|ui| {
                    ui.label("背景颜色:");
                    ui.color_edit_button_srgba(&mut self.state.background_color);
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("主题模式:");
                    ui.selectable_value(&mut self.state.theme_mode, ThemeMode::System, "跟随系统");
                    ui.selectable_value(&mut self.state.theme_mode, ThemeMode::Light, "浅色模式");
                    ui.selectable_value(&mut self.state.theme_mode, ThemeMode::Dark, "深色模式");
                });
            });

            ui.collapsing("绘制", |ui| {
                ui.horizontal(|ui| {
                    ui.label("插值频率:");
                    ui.add(egui::Slider::new(
                        &mut self.state.interpolation_frequency,
                        0.0..=1.0,
                    ));
                });

                ui.horizontal(|ui| {
                    ui.label("快捷颜色管理:");
                    if ui.button("编辑快捷颜色").clicked() {
                        self.state.show_quick_color_editor = true;
                    }
                });

                // Quick color editor window
                if self.state.show_quick_color_editor {
                    let content_rect = ui.ctx().available_rect();
                    let center_pos = content_rect.center();

                    egui::Window::new("编辑快捷颜色")
                        .collapsible(false)
                        .resizable(false)
                        .pivot(egui::Align2::CENTER_CENTER)
                        .default_pos([center_pos.x, center_pos.y])
                        .show(ui.ctx(), |ui| {
                            ui.label("当前快捷颜色:");
                            ui.separator();

                            let mut color_index_to_remove = None;
                            for (index, color) in self.state.quick_colors.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    let mut temp_color = *color;
                                    ui.color_edit_button_srgba(&mut temp_color);
                                    if ui.button("删除").clicked() {
                                        color_index_to_remove = Some(index);
                                    }
                                });
                            }

                            if let Some(index) = color_index_to_remove {
                                self.state.quick_colors.remove(index);
                            }

                            ui.separator();

                            ui.horizontal(|ui| {
                                ui.label("新颜色:");
                                ui.color_edit_button_srgba(&mut self.state.new_quick_color);
                                if ui.button("添加").clicked() {
                                    self.state.quick_colors.push(self.state.new_quick_color);
                                    self.state.new_quick_color = Color32::WHITE;
                                }
                            });

                            ui.separator();

                            ui.horizontal(|ui| {
                                if ui.button("完成").clicked() {
                                    self.state.show_quick_color_editor = false;
                                }
                                if ui.button("重置").clicked() {
                                    self.state.show_quick_color_editor = false;
                                    self.state.quick_colors = vec![
                                        Color32::from_rgb(255, 0, 0),   // 红色
                                        Color32::from_rgb(255, 255, 0), // 黄色
                                        Color32::from_rgb(0, 255, 0),   // 绿色
                                    ];
                                }
                            });
                        });
                }
            });

            ui.collapsing("性能", |ui| {
                ui.horizontal(|ui| {
                    ui.label("窗口模式:");
                    if ui
                        .selectable_value(
                            &mut self.state.window_mode,
                            WindowMode::Windowed,
                            "窗口化",
                        )
                        .clicked()
                    {
                        ui.ctx()
                            .send_viewport_cmd(ViewportCommand::Fullscreen(false));
                    }
                    if ui
                        .selectable_value(
                            &mut self.state.window_mode,
                            WindowMode::Fullscreen,
                            "全屏",
                        )
                        .clicked()
                    {
                        println!("not supported in eframe")
                    }
                    if ui
                        .selectable_value(
                            &mut self.state.window_mode,
                            WindowMode::BorderlessFullscreen,
                            "无边框全屏",
                        )
                        .clicked()
                    {
                        ui.ctx()
                            .send_viewport_cmd(ViewportCommand::Fullscreen(true));
                    }
                });

                // Display mode selection (only available in fullscreen mode)
                // ui.horizontal(|ui| {
                //     ui.label("显示模式:");

                //     let mut current_selection: usize =
                //         self.state.selected_video_mode_index.unwrap_or(0);

                //     if self.state.window_mode == WindowMode::Fullscreen {
                //         let video_modes = self.state.available_video_modes.clone();

                //         if let Some(mode) = video_modes.get(current_selection) {
                //             let mode_text = format!(
                //                 "{}x{} @ {}Hz",
                //                 mode.size().width,
                //                 mode.size().height,
                //                 mode.refresh_rate_millihertz() as f32 / 1000.0
                //             );
                //             ui.label(mode_text);
                //         }

                //         egui::ComboBox::from_id_salt("video_mode_selection").show_ui(ui, |ui| {
                //             for (index, mode) in video_modes.iter().enumerate() {
                //                 let mode_text = format!(
                //                     "{}x{} @ {}Hz",
                //                     mode.size().width,
                //                     mode.size().height,
                //                     mode.refresh_rate_millihertz() as f32 / 1000.0
                //                 );
                //                 ui.selectable_value(&mut current_selection, index, mode_text);
                //             }
                //         });
                //     } else {
                //         if let Some(mode) = self.state.available_video_modes.get(current_selection)
                //         {
                //             let mode_text = format!(
                //                 "{}x{} @ {}Hz",
                //                 mode.size().width,
                //                 mode.size().height,
                //                 mode.refresh_rate_millihertz() as f32 / 1000.0
                //             );
                //             ui.label(mode_text);
                //         }
                //     }

                //     self.state.selected_video_mode_index = Some(current_selection);

                //     if self.state.window_mode == WindowMode::Fullscreen {
                //         self.state.window_mode_changed = true;
                //     }
                // });

                // Vertical sync mode selection
                ui.horizontal(|ui| {
                    ui.label("垂直同步:");
                    let old_present_mode = self.state.present_mode;
                    let present_mode_changed = ui
                        .selectable_value(
                            &mut self.state.present_mode,
                            PresentMode::AAutoVsync,
                            "开 (自动) | AutoVsync",
                        )
                        .changed()
                        || ui
                            .selectable_value(
                                &mut self.state.present_mode,
                                PresentMode::AutoNoVsync,
                                "关 (自动) | AutoNoVsync",
                            )
                            .changed()
                        || ui
                            .selectable_value(
                                &mut self.state.present_mode,
                                PresentMode::Fifo,
                                "开 | Fifo",
                            )
                            .changed()
                        || ui
                            .selectable_value(
                                &mut self.state.present_mode,
                                PresentMode::FifoRelaxed,
                                "自适应 | FifoRelaxed",
                            )
                            .changed()
                        || ui
                            .selectable_value(
                                &mut self.state.present_mode,
                                PresentMode::Immediate,
                                "关 | Immediate",
                            )
                            .changed()
                        || ui
                            .selectable_value(
                                &mut self.state.present_mode,
                                PresentMode::Mailbox,
                                "开 (快速) | Mailbox",
                            )
                            .changed();

                    if present_mode_changed && self.state.present_mode != old_present_mode {
                        self.state.present_mode_changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("渲染更新模式:");
                    ui.selectable_value(&mut self.state.render_update_mode, RenderUpdateMode::Reactive, "Reactive");
                    ui.selectable_value(&mut self.state.render_update_mode, RenderUpdateMode::Continuous, "Continuous");
                });
            });

            ui.collapsing("调试", |ui| {
                ui.horizontal(|ui| {
                    ui.label("引发异常:");
                    if ui.button("OK").clicked() {
                        panic!("test panic")
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("显示 FPS:");
                    ui.checkbox(&mut self.state.show_fps, "启用");
                });

                ui.horizontal(|ui| {
                    ui.label("显示触控点:");
                    ui.checkbox(&mut self.state.show_touch_points, "启用");
                });

                ui.horizontal(|ui| {
                    ui.label("压力测试:");
                    if ui.button("OK").clicked() {
                        let stress_color = Color32::from_rgb(255, 0, 0);
                        let stress_width = 3.0;

                        for i in 0..1000 {
                            let mut points = Vec::new();
                            let mut widths = Vec::new();

                            let num_points = 100;

                            let start_x = (i as f32 % 20.0) * 50.0;
                            let start_y = ((i as f32 / 20.0).floor() % 15.0) * 50.0;

                            for j in 0..num_points {
                                let x = start_x + (j as f32 * 10.0);
                                let y = start_y + (j as f32 * 5.0);

                                points.push(Pos2::new(x, y));
                                widths.push(stress_width);
                            }

                            let stroke = crate::state::CanvasStroke {
                                points,
                                widths,
                                color: stress_color,
                                base_width: stress_width,
                            };

                            self.state.canvas_objects.push(CanvasObject::Stroke(stroke));
                        }
                    }
                });
            });
        }

        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("退出").clicked() {
                ui.ctx().send_viewport_cmd(ViewportCommand::Close);
            }
            if self.state.show_fps {
                ui.label(format!(
                    "FPS: {}",
                    self.state.fps_counter.current_fps.to_string()
                ));
            }
        });
    }

    fn render_canvas(&mut self, ui: &mut egui::Ui) {
        let (rect, response) =
            ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());

        let painter = ui.painter();

        // Draw background
        painter.rect_filled(rect, 0.0, self.state.background_color);

        // Draw all objects
        for (i, object) in self.state.canvas_objects.iter().enumerate() {
            let selected = self.state.selected_object == Some(i);
            object.draw(painter, selected);
        }

        // Draw currently drawing strokes
        for (_touch_id, active_stroke) in &self.state.active_strokes {
            if active_stroke.points.len() >= 2
                && active_stroke.widths.len() == active_stroke.points.len()
            {
                let all_same_width = active_stroke
                    .widths
                    .windows(2)
                    .all(|w| (w[0] - w[1]).abs() < 0.01);

                if all_same_width && active_stroke.points.len() == 2 {
                    painter.line_segment(
                        [active_stroke.points[0], active_stroke.points[1]],
                        Stroke::new(active_stroke.widths[0], self.state.brush_color),
                    );
                } else if all_same_width {
                    let path = egui::epaint::PathShape::line(
                        active_stroke.points.clone(),
                        Stroke::new(active_stroke.widths[0], self.state.brush_color),
                    );
                    painter.add(Shape::Path(path));
                } else {
                    for i in 0..active_stroke.points.len() - 1 {
                        let avg_width =
                            (active_stroke.widths[i] + active_stroke.widths[i + 1]) / 2.0;
                        painter.line_segment(
                            [active_stroke.points[i], active_stroke.points[i + 1]],
                            Stroke::new(avg_width, self.state.brush_color),
                        );
                    }
                }
            }
        }

        // Draw size preview circle
        if self.state.show_size_preview {
            let content_rect = ui.ctx().available_rect();
            let pos = content_rect.center();
            AppUtils::draw_size_preview(
                painter,
                pos,
                match self.state.current_tool {
                    CanvasTool::Brush => self.state.brush_width,
                    CanvasTool::ObjectEraser | CanvasTool::PixelEraser => self.state.eraser_size,
                    _ => 10.0, // fallback
                },
            );
        }

        if self.state.show_touch_points {
            for (id, pos) in &self.state.touch_points {
                painter.circle_filled(
                    *pos,
                    15.0,
                    Color32::from_rgba_unmultiplied(255, 255, 255, 180),
                );
                painter.circle_stroke(*pos, 15.0, Stroke::new(2.0, Color32::BLUE));

                let text_galley = painter.layout_no_wrap(
                    format!("{}", id),
                    egui::FontId::proportional(14.0),
                    Color32::BLACK,
                );
                let text_pos = Pos2::new(
                    pos.x - text_galley.size().x / 2.0,
                    pos.y - text_galley.size().y / 2.0,
                );
                let text_shape = egui::epaint::TextShape {
                    pos: text_pos,
                    galley: text_galley,
                    underline: egui::Stroke::NONE,
                    override_text_color: None,
                    angle: 0.0,
                    fallback_color: Color32::BLACK,
                    opacity_factor: 1.0,
                };
                painter.add(text_shape);
            }
        }

        // Draw resize and rotation anchors
        if let Some(selected_idx) = self.state.selected_object {
            if let Some(object) = self.state.canvas_objects.get(selected_idx) {
                let object_rect = match object {
                    CanvasObject::Image(img) => egui::Rect::from_min_size(img.pos, img.size),
                    CanvasObject::Text(text) => {
                        let text_galley = painter.layout_no_wrap(
                            text.text.clone(),
                            egui::FontId::proportional(text.font_size),
                            text.color,
                        );
                        let text_size = text_galley.size();
                        egui::Rect::from_min_size(text.pos, text_size)
                    }
                    CanvasObject::Shape(shape) => AppUtils::calculate_shape_bounding_box(shape),
                    CanvasObject::Stroke(_) => {
                        return;
                    }
                };

                AppUtils::draw_resize_and_rotation_anchors(
                    &painter,
                    object_rect,
                    self.state.resize_anchor_hovered,
                    self.state.rotation_anchor_hovered,
                );
            }
        }

        // Handle mouse input
        let pointer_pos = response.interact_pointer_pos();

        match self.state.current_tool {
            CanvasTool::Insert | CanvasTool::Settings => {}

            CanvasTool::Select => {
                if let Some(pos) = pointer_pos {
                    self.state.drag_start_pos = Some(pos);

                    let mut hit = false;
                    for object in &self.state.canvas_objects {
                        if let CanvasObject::Image(img) = object {
                            if egui::Rect::from_min_size(img.pos, img.size).contains(pos) {
                                hit = true;
                                break;
                            }
                        }
                    }
                    if !hit {
                        for object in &self.state.canvas_objects {
                            if let CanvasObject::Stroke(stroke) = object {
                                if AppUtils::point_intersects_stroke(pos, stroke, 10.0) {
                                    hit = true;
                                    break;
                                }
                            }
                        }
                    }
                    if !hit {
                        self.state.selected_object = None;
                    }

                    if let Some(selected_idx) = self.state.selected_object {
                        if let Some(object) = self.state.canvas_objects.get(selected_idx) {
                            let object_rect = match object {
                                CanvasObject::Image(img) => {
                                    Some(egui::Rect::from_min_size(img.pos, img.size))
                                }
                                CanvasObject::Text(text) => {
                                    let text_galley = painter.layout_no_wrap(
                                        text.text.clone(),
                                        egui::FontId::proportional(text.font_size),
                                        text.color,
                                    );
                                    let text_size = text_galley.size();
                                    Some(egui::Rect::from_min_size(text.pos, text_size))
                                }
                                CanvasObject::Shape(shape) => {
                                    Some(AppUtils::calculate_shape_bounding_box(shape))
                                }
                                CanvasObject::Stroke(_) => None,
                            };

                            if let Some(rect) = object_rect {
                                let resize_anchors = [
                                    (ResizeAnchor::TopLeft, rect.left_top()),
                                    (ResizeAnchor::TopRight, rect.right_top()),
                                    (ResizeAnchor::BottomLeft, rect.left_bottom()),
                                    (ResizeAnchor::BottomRight, rect.right_bottom()),
                                    (ResizeAnchor::Top, Pos2::new(rect.center().x, rect.min.y)),
                                    (ResizeAnchor::Bottom, Pos2::new(rect.center().x, rect.max.y)),
                                    (ResizeAnchor::Left, Pos2::new(rect.min.x, rect.center().y)),
                                    (ResizeAnchor::Right, Pos2::new(rect.max.x, rect.center().y)),
                                ];

                                let mut found_resize_anchor = None;
                                for (anchor_type, anchor_pos) in resize_anchors {
                                    if pos.distance(anchor_pos) <= 15.0 {
                                        found_resize_anchor = Some(anchor_type);
                                        break;
                                    }
                                }

                                self.state.resize_anchor_hovered = found_resize_anchor;

                                let rotation_anchor_pos =
                                    Pos2::new(rect.center().x, rect.min.y - 30.0);
                                self.state.rotation_anchor_hovered =
                                    pos.distance(rotation_anchor_pos) <= 15.0;
                            } else {
                                self.state.resize_anchor_hovered = None;
                                self.state.rotation_anchor_hovered = false;
                            }
                        } else {
                            self.state.resize_anchor_hovered = None;
                            self.state.rotation_anchor_hovered = false;
                        }
                    } else {
                        self.state.resize_anchor_hovered = None;
                        self.state.rotation_anchor_hovered = false;
                    }

                    if response.drag_started() {
                        if let Some(pos) = pointer_pos {
                            self.state.drag_start_pos = Some(pos);

                            let mut hit = false;
                            for object in &self.state.canvas_objects {
                                if let CanvasObject::Image(img) = object {
                                    if egui::Rect::from_min_size(img.pos, img.size).contains(pos) {
                                        hit = true;
                                        break;
                                    }
                                }
                            }
                            if !hit {
                                for object in &self.state.canvas_objects {
                                    if let CanvasObject::Stroke(stroke) = object {
                                        if AppUtils::point_intersects_stroke(pos, stroke, 10.0) {
                                            hit = true;
                                            break;
                                        }
                                    }
                                }
                            }
                            if !hit {
                                self.state.selected_object = None;
                            }

                            if let Some(selected_idx) = self.state.selected_object {
                                if let Some(object) = self.state.canvas_objects.get(selected_idx) {
                                    let object_rect = match object {
                                        CanvasObject::Image(img) => {
                                            Some(egui::Rect::from_min_size(img.pos, img.size))
                                        }
                                        CanvasObject::Text(text) => {
                                            let text_galley = painter.layout_no_wrap(
                                                text.text.clone(),
                                                egui::FontId::proportional(text.font_size),
                                                text.color,
                                            );
                                            let text_size = text_galley.size();
                                            Some(egui::Rect::from_min_size(text.pos, text_size))
                                        }
                                        CanvasObject::Shape(shape) => {
                                            Some(AppUtils::calculate_shape_bounding_box(shape))
                                        }
                                        CanvasObject::Stroke(_) => None,
                                    };

                                    if let Some(rect) = object_rect {
                                        if let Some(anchor) = self.state.resize_anchor_hovered {
                                            self.state.resize_operation = Some(ResizeOperation {
                                                anchor,
                                                start_pos: pos,
                                                start_size: rect.size(),
                                                start_object_pos: rect.min,
                                            });
                                        } else if self.state.rotation_anchor_hovered {
                                            self.state.rotation_operation =
                                                Some(RotationOperation {
                                                    start_pos: pos,
                                                    start_angle: 0.0,
                                                    center: rect.center(),
                                                });

                                            if let Some(CanvasObject::Shape(shape)) =
                                                self.state.canvas_objects.get(selected_idx)
                                            {
                                                if let Some(op) =
                                                    self.state.rotation_operation.as_mut()
                                                {
                                                    op.start_angle = shape.rotation;
                                                }
                                            }
                                        } else if rect.contains(pos) {
                                        } else {
                                            self.state.selected_object = None;
                                        }
                                    }
                                }
                            } else {
                                self.state.selected_object = None;

                                for (i, object) in
                                    self.state.canvas_objects.iter().enumerate().rev()
                                {
                                    match object {
                                        CanvasObject::Image(img) => {
                                            let img_rect =
                                                egui::Rect::from_min_size(img.pos, img.size);
                                            if img_rect.contains(pos) {
                                                self.state.selected_object = Some(i);
                                                break;
                                            }
                                        }
                                        CanvasObject::Text(text) => {
                                            let text_galley = painter.layout_no_wrap(
                                                text.text.clone(),
                                                egui::FontId::proportional(text.font_size),
                                                text.color,
                                            );
                                            let text_size = text_galley.size();
                                            let text_rect =
                                                egui::Rect::from_min_size(text.pos, text_size);
                                            if text_rect.contains(pos) {
                                                self.state.selected_object = Some(i);
                                                break;
                                            }
                                        }
                                        CanvasObject::Shape(shape) => {
                                            let shape_rect =
                                                AppUtils::calculate_shape_bounding_box(shape);
                                            if shape_rect.contains(pos) {
                                                self.state.selected_object = Some(i);
                                                break;
                                            }
                                        }
                                        CanvasObject::Stroke(stroke) => {
                                            if AppUtils::point_intersects_stroke(pos, stroke, 10.0)
                                            {
                                                self.state.selected_object = Some(i);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if response.clicked() {
                        if let Some(pos) = pointer_pos {
                            let mut hit = false;
                            for object in &self.state.canvas_objects {
                                if let CanvasObject::Image(img) = object {
                                    if egui::Rect::from_min_size(img.pos, img.size).contains(pos) {
                                        hit = true;
                                        break;
                                    }
                                }
                            }
                            if !hit {
                                for object in &self.state.canvas_objects {
                                    if let CanvasObject::Stroke(stroke) = object {
                                        if AppUtils::point_intersects_stroke(pos, stroke, 10.0) {
                                            hit = true;
                                            break;
                                        }
                                    }
                                }
                            }
                            if !hit {
                                self.state.selected_object = None;
                            }
                        }
                    } else if response.dragged() {
                        if let Some(pos) = pointer_pos {
                            if let Some(resize_op) = self.state.resize_operation {
                                if let Some(selected_idx) = self.state.selected_object {
                                    if let Some(object) =
                                        self.state.canvas_objects.get_mut(selected_idx)
                                    {
                                        let delta = pos - resize_op.start_pos;

                                        match object {
                                            CanvasObject::Image(img) => {
                                                let mut new_size = resize_op.start_size;
                                                let mut new_pos = resize_op.start_object_pos;

                                                match resize_op.anchor {
                                                    ResizeAnchor::TopLeft => {
                                                        new_size.x = (resize_op.start_size.x
                                                            - delta.x)
                                                            .max(20.0);
                                                        new_size.y = (resize_op.start_size.y
                                                            - delta.y)
                                                            .max(20.0);
                                                        new_pos.x =
                                                            resize_op.start_object_pos.x + delta.x;
                                                        new_pos.y =
                                                            resize_op.start_object_pos.y + delta.y;
                                                    }
                                                    ResizeAnchor::TopRight => {
                                                        new_size.x = (resize_op.start_size.x
                                                            + delta.x)
                                                            .max(20.0);
                                                        new_size.y = (resize_op.start_size.y
                                                            - delta.y)
                                                            .max(20.0);
                                                        new_pos.y =
                                                            resize_op.start_object_pos.y + delta.y;
                                                    }
                                                    ResizeAnchor::BottomLeft => {
                                                        new_size.x = (resize_op.start_size.x
                                                            - delta.x)
                                                            .max(20.0);
                                                        new_size.y = (resize_op.start_size.y
                                                            + delta.y)
                                                            .max(20.0);
                                                        new_pos.x =
                                                            resize_op.start_object_pos.x + delta.x;
                                                    }
                                                    ResizeAnchor::BottomRight => {
                                                        new_size.x = (resize_op.start_size.x
                                                            + delta.x)
                                                            .max(20.0);
                                                        new_size.y = (resize_op.start_size.y
                                                            + delta.y)
                                                            .max(20.0);
                                                    }
                                                    ResizeAnchor::Top => {
                                                        new_size.y = (resize_op.start_size.y
                                                            - delta.y)
                                                            .max(20.0);
                                                        new_pos.y =
                                                            resize_op.start_object_pos.y + delta.y;
                                                    }
                                                    ResizeAnchor::Bottom => {
                                                        new_size.y = (resize_op.start_size.y
                                                            + delta.y)
                                                            .max(20.0);
                                                    }
                                                    ResizeAnchor::Left => {
                                                        new_size.x = (resize_op.start_size.x
                                                            - delta.x)
                                                            .max(20.0);
                                                        new_pos.x =
                                                            resize_op.start_object_pos.x + delta.x;
                                                    }
                                                    ResizeAnchor::Right => {
                                                        new_size.x = (resize_op.start_size.x
                                                            + delta.x)
                                                            .max(20.0);
                                                    }
                                                }

                                                if img.aspect_ratio > 0.0 {
                                                    let target_aspect = img.aspect_ratio;
                                                    let current_aspect = new_size.x / new_size.y;

                                                    if current_aspect.abs() > 0.01 {
                                                        if current_aspect > target_aspect {
                                                            new_size.x = new_size.y * target_aspect;
                                                        } else {
                                                            new_size.y = new_size.x / target_aspect;
                                                        }
                                                    }
                                                }

                                                img.pos = new_pos;
                                                img.size = new_size;
                                            }
                                            CanvasObject::Text(text) => match resize_op.anchor {
                                                ResizeAnchor::TopLeft
                                                | ResizeAnchor::BottomRight => {
                                                    text.font_size =
                                                        (resize_op.start_size.x + delta.x).max(8.0);
                                                }
                                                _ => {}
                                            },
                                            CanvasObject::Shape(shape) => {
                                                let delta = pos - resize_op.start_pos;

                                                match resize_op.anchor {
                                                    ResizeAnchor::TopLeft
                                                    | ResizeAnchor::BottomRight => {
                                                        shape.size = (resize_op.start_size.x
                                                            + delta.x)
                                                            .max(10.0);
                                                    }
                                                    ResizeAnchor::TopRight
                                                    | ResizeAnchor::BottomLeft => {
                                                        shape.size = (resize_op.start_size.x
                                                            - delta.x)
                                                            .max(10.0);
                                                    }
                                                    ResizeAnchor::Top | ResizeAnchor::Bottom => {
                                                        shape.size = (resize_op.start_size.y
                                                            + delta.y)
                                                            .max(10.0);
                                                    }
                                                    ResizeAnchor::Left | ResizeAnchor::Right => {
                                                        shape.size = (resize_op.start_size.x
                                                            + delta.x)
                                                            .max(10.0);
                                                    }
                                                }
                                            }
                                            CanvasObject::Stroke(_) => {}
                                        }
                                    }
                                }
                            } else if let Some(rotate_op) = self.state.rotation_operation {
                                if let Some(selected_idx) = self.state.selected_object {
                                    if let Some(object) =
                                        self.state.canvas_objects.get_mut(selected_idx)
                                    {
                                        let center = rotate_op.center;
                                        let current_dir = pos - center;
                                        let start_dir = rotate_op.start_pos - center;

                                        let current_angle = current_dir.y.atan2(current_dir.x);
                                        let start_angle = start_dir.y.atan2(start_dir.x);

                                        let angle_delta = current_angle - start_angle;

                                        match object {
                                            CanvasObject::Shape(shape) => {
                                                shape.rotation =
                                                    rotate_op.start_angle + angle_delta;
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            } else if let (Some(start_pos), Some(selected_idx)) =
                                (self.state.drag_start_pos, self.state.selected_object)
                            {
                                let delta = pos - start_pos;
                                self.state.drag_start_pos = Some(pos);

                                if let Some(object) =
                                    self.state.canvas_objects.get_mut(selected_idx)
                                {
                                    match object {
                                        CanvasObject::Image(img) => {
                                            img.pos += delta;
                                        }
                                        CanvasObject::Stroke(stroke) => {
                                            for p in &mut stroke.points {
                                                *p += delta;
                                            }
                                        }
                                        CanvasObject::Text(text) => {
                                            text.pos += delta;
                                        }
                                        CanvasObject::Shape(shape) => {
                                            shape.pos += delta;
                                        }
                                    }
                                }
                            }
                        }
                    } else if response.drag_stopped() {
                        self.state.resize_operation = None;
                        self.state.rotation_operation = None;
                        self.state.drag_start_pos = None;
                    }
                }
            }

            CanvasTool::ObjectEraser => {
                if response.drag_started() || response.clicked() || response.dragged() {
                    if let Some(pos) = pointer_pos {
                        AppUtils::draw_size_preview(painter, pos, self.state.eraser_size);

                        let mut to_remove = Vec::new();

                        for (i, object) in self.state.canvas_objects.iter().enumerate().rev() {
                            match object {
                                CanvasObject::Image(img) => {
                                    let img_rect = egui::Rect::from_min_size(img.pos, img.size);
                                    if img_rect.contains(pos) {
                                        to_remove.push(i);
                                    }
                                }
                                CanvasObject::Text(text) => {
                                    let text_galley = painter.layout_no_wrap(
                                        text.text.clone(),
                                        egui::FontId::proportional(text.font_size),
                                        text.color,
                                    );
                                    let text_size = text_galley.size();
                                    let text_rect = egui::Rect::from_min_size(text.pos, text_size);
                                    if text_rect.contains(pos) {
                                        to_remove.push(i);
                                    }
                                }
                                CanvasObject::Shape(shape) => {
                                    let shape_rect = AppUtils::calculate_shape_bounding_box(shape);
                                    if shape_rect.contains(pos) {
                                        to_remove.push(i);
                                    }
                                }
                                CanvasObject::Stroke(stroke) => {
                                    if AppUtils::point_intersects_stroke(
                                        pos,
                                        stroke,
                                        self.state.eraser_size,
                                    ) {
                                        to_remove.push(i);
                                    }
                                }
                            }
                        }

                        for i in to_remove {
                            self.state.canvas_objects.remove(i);
                        }
                    }
                }
            }

            CanvasTool::PixelEraser => {
                if response.dragged() || response.clicked() {
                    if let Some(pos) = pointer_pos {
                        AppUtils::draw_size_preview(painter, pos, self.state.eraser_size);

                        let eraser_radius = self.state.eraser_size / 2.0;
                        let mut new_strokes = Vec::new();

                        for object in &self.state.canvas_objects {
                            if let CanvasObject::Stroke(stroke) = object {
                                if stroke.points.len() < 2 {
                                    continue;
                                }

                                let mut current_points = Vec::new();
                                let mut current_widths = Vec::new();

                                current_points.push(stroke.points[0]);
                                if !stroke.widths.is_empty() {
                                    current_widths.push(stroke.widths[0]);
                                }

                                for i in 0..stroke.points.len() - 1 {
                                    let p1 = stroke.points[i];
                                    let p2 = stroke.points[i + 1];
                                    let segment_width = if i < stroke.widths.len() {
                                        stroke.widths[i]
                                    } else {
                                        stroke.widths[0]
                                    };

                                    let dist =
                                        AppUtils::point_to_line_segment_distance(pos, p1, p2);

                                    if dist > eraser_radius + segment_width / 2.0 {
                                        current_points.push(p2);
                                        if i + 1 < stroke.widths.len() {
                                            current_widths.push(stroke.widths[i + 1]);
                                        } else if !stroke.widths.is_empty() {
                                            current_widths
                                                .push(stroke.widths[stroke.widths.len() - 1]);
                                        }
                                    } else {
                                        if current_points.len() >= 2 {
                                            new_strokes.push(crate::state::CanvasStroke {
                                                points: current_points.clone(),
                                                widths: current_widths.clone(),
                                                color: stroke.color,
                                                base_width: stroke.base_width,
                                            });
                                        }
                                        current_points = Vec::new();
                                        current_widths = Vec::new();
                                    }
                                }

                                if current_points.len() >= 2 {
                                    new_strokes.push(crate::state::CanvasStroke {
                                        points: current_points,
                                        widths: current_widths,
                                        color: stroke.color,
                                        base_width: stroke.base_width,
                                    });
                                }
                            } else {
                                if let CanvasObject::Stroke(stroke) = object {
                                    new_strokes.push(stroke.clone());
                                }
                            }
                        }

                        self.state.canvas_objects = self
                            .state
                            .canvas_objects
                            .iter()
                            .filter_map(|obj| {
                                if let CanvasObject::Stroke(_) = obj {
                                    None
                                } else {
                                    Some(obj.clone())
                                }
                            })
                            .collect();

                        for stroke in new_strokes {
                            self.state.canvas_objects.push(CanvasObject::Stroke(stroke));
                        }
                    }
                }
            }

            CanvasTool::Brush => {
                if response.drag_started() {
                    if let Some(pos) = pointer_pos {
                        if pos.x >= rect.min.x
                            && pos.x <= rect.max.x
                            && pos.y >= rect.min.y
                            && pos.y <= rect.max.y
                        {
                            self.state.is_drawing = true;
                            let start_time = Instant::now();
                            let width = AppUtils::calculate_dynamic_width(
                                self.state.brush_width,
                                self.state.dynamic_brush_width_mode,
                                0,
                                1,
                                None,
                            );

                            let touch_id = 0;
                            self.state.active_strokes.insert(
                                touch_id,
                                crate::state::ActiveStroke {
                                    points: vec![pos],
                                    widths: vec![width],
                                    times: vec![0.0],
                                    start_time,
                                },
                            );
                        }
                    }
                } else if response.dragged() {
                    if self.state.is_drawing {
                        if let Some(pos) = pointer_pos {
                            let touch_id = 0;
                            if let Some(active_stroke) =
                                self.state.active_strokes.get_mut(&touch_id)
                            {
                                let current_time = active_stroke.start_time.elapsed().as_secs_f64();

                                if active_stroke.points.is_empty()
                                    || active_stroke.points.last().unwrap().distance(pos) > 1.0
                                {
                                    let speed = if active_stroke.points.len() > 0
                                        && active_stroke.times.len() > 0
                                    {
                                        let last_time = active_stroke.times.last().unwrap();
                                        let time_delta =
                                            ((current_time - last_time) as f32).max(0.001);
                                        let distance =
                                            active_stroke.points.last().unwrap().distance(pos);
                                        Some(distance / time_delta)
                                    } else {
                                        None
                                    };

                                    active_stroke.points.push(pos);
                                    active_stroke.times.push(current_time);

                                    let width = AppUtils::calculate_dynamic_width(
                                        self.state.brush_width,
                                        self.state.dynamic_brush_width_mode,
                                        active_stroke.points.len() - 1,
                                        active_stroke.points.len(),
                                        speed,
                                    );
                                    active_stroke.widths.push(width);
                                }
                            }
                        }
                    }
                } else if response.drag_stopped() {
                    if self.state.is_drawing {
                        let touch_id = 0;
                        if let Some(active_stroke) = self.state.active_strokes.remove(&touch_id) {
                            if active_stroke.points.len() > 1
                                && active_stroke.widths.len() == active_stroke.points.len()
                            {
                                let final_points = if self.state.stroke_smoothing {
                                    AppUtils::apply_stroke_smoothing(&active_stroke.points)
                                } else {
                                    active_stroke.points
                                };

                                let (interpolated_points, interpolated_widths) =
                                    AppUtils::apply_point_interpolation(
                                        &final_points,
                                        &active_stroke.widths,
                                        self.state.interpolation_frequency,
                                    );

                                self.state.canvas_objects.push(CanvasObject::Stroke(
                                    crate::state::CanvasStroke {
                                        points: interpolated_points,
                                        widths: interpolated_widths,
                                        color: self.state.brush_color,
                                        base_width: self.state.brush_width,
                                    },
                                ));
                            }
                        }

                        self.state.is_drawing = !self.state.active_strokes.is_empty();
                    }
                }

                if response.hovered() && self.state.is_drawing {
                    if let Some(pos) = pointer_pos {
                        let touch_id = 0;
                        if let Some(active_stroke) = self.state.active_strokes.get_mut(&touch_id) {
                            let current_time = active_stroke.start_time.elapsed().as_secs_f64();

                            if active_stroke.points.is_empty()
                                || active_stroke.points.last().unwrap().distance(pos) > 1.0
                            {
                                let speed = if active_stroke.points.len() > 0
                                    && active_stroke.times.len() > 0
                                {
                                    let last_time = active_stroke.times.last().unwrap();
                                    let time_delta = ((current_time - last_time) as f32).max(0.001);
                                    let distance =
                                        active_stroke.points.last().unwrap().distance(pos);
                                    Some(distance / time_delta)
                                } else {
                                    None
                                };

                                active_stroke.points.push(pos);
                                active_stroke.times.push(current_time);

                                let width = AppUtils::calculate_dynamic_width(
                                    self.state.brush_width,
                                    self.state.dynamic_brush_width_mode,
                                    active_stroke.points.len() - 1,
                                    active_stroke.points.len(),
                                    speed,
                                );
                                active_stroke.widths.push(width);
                            }
                        }
                    }
                }
            }
        }
    }
}
