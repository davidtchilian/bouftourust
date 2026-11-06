use super::*;

fn slot(
    ui: &mut egui::Ui,
    art: &mut Art,
    p: &Profile,
    d: &GameData,
    actions: &mut Actions,
    slot: &str,
) {
    ui.vertical(|ui| {
        ui.small(&d.slot_names[slot]);
        let response = egui::Frame::new()
            .fill(egui::Color32::from_rgb(17, 30, 33))
            .stroke(egui::Stroke::new(
                1_f32,
                egui::Color32::from_rgb(52, 72, 74),
            ))
            .corner_radius(9)
            .inner_margin(4.)
            .show(ui, |ui| {
                if let Some(id) = p.worn(slot) {
                    art.icon(ui, &d.items[id].icon, egui::vec2(46., 46.))
                        .on_hover_ui(|ui| {
                            item_tooltip(ui, &d.items[id], d);
                            ui.small("Cliquer pour retirer");
                        })
                } else {
                    ui.add_sized(
                        [46., 46.],
                        egui::Button::new(egui::RichText::new("Vide").small()).frame(false),
                    )
                }
            })
            .inner;
        if response.clicked() && p.worn(slot).is_some() {
            actions.0.push(Action::Unequip(slot.into()));
        }
    });
}
fn portrait(ui: &mut egui::Ui, art: &mut Art, state: &mut UiState, p: &Profile, width: f32) {
    egui::Frame::new()
        .fill(egui::Color32::from_rgb(27, 41, 44))
        .stroke(egui::Stroke::new(
            1_f32,
            egui::Color32::from_rgb(52, 72, 74),
        ))
        .corner_radius(12)
        .inner_margin(8.)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2((width - 16.).max(55.), 264.),
                    egui::Sense::hover(),
                );
                let id = format!("{}.idle.{}", p.class_id, DIRECTIONS[state.direction]);
                let pose = &art.manifest.composition.actors[&id];
                let mut bounds = [
                    f32::INFINITY,
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                    f32::NEG_INFINITY,
                ];
                for part in &pose.frames[0].layers {
                    if part.hide_with_hat && p.worn("hat").is_some() {
                        continue;
                    }
                    let resolved = part.asset_id.as_deref().or_else(|| {
                        part.slot
                            .as_ref()
                            .and_then(|slot| p.worn(slot))
                            .and_then(|item| art.manifest.composition.accessories.get(item))
                            .and_then(|a| a.views.get(part.label.as_deref().unwrap_or("")))
                            .map(String::as_str)
                    });
                    if let Some(asset) = resolved.and_then(|id| art.manifest.assets.get(id)) {
                        let x = -asset.pivot[0] * asset.size[0];
                        let y = -asset.pivot[1] * asset.size[1];
                        let [a, b, c, d, tx, ty] = part.matrix;
                        for (x, y) in [
                            (x, y),
                            (x + asset.size[0], y),
                            (x, y + asset.size[1]),
                            (x + asset.size[0], y + asset.size[1]),
                        ] {
                            let px = a * x + c * y + tx;
                            let py = b * x + d * y + ty;
                            bounds[0] = bounds[0].min(px);
                            bounds[1] = bounds[1].min(py);
                            bounds[2] = bounds[2].max(px);
                            bounds[3] = bounds[3].max(py);
                        }
                    }
                }
                let scale = (rect.width() / (bounds[2] - bounds[0] + 8.))
                    .min(rect.height() / (bounds[3] - bounds[1] + 12.));
                let mirror = if pose.flip_x { -1. } else { 1. };
                let origin = rect.center()
                    - egui::vec2(
                        mirror * (bounds[0] + bounds[2]) / 2.,
                        (bounds[1] + bounds[3]) / 2.,
                    ) * scale;
                art.actor(ui.painter(), &id, p, 0., origin, scale, None, None, true);
                ui.vertical_centered(|ui| {
                    if ui.button("Tourner").clicked() {
                        state.direction = (state.direction + 1) % 8;
                    }
                });
            });
        });
}
fn sheet(
    ui: &mut egui::Ui,
    art: &mut Art,
    state: &mut UiState,
    p: &Profile,
    d: &GameData,
    actions: &mut Actions,
) {
    ui.heading(&d.classes[&p.class_id].name);
    ui.add_space(6.);
    let width = ui.available_width();
    let portrait_width = (width - 140.).max(90.);
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(58.);
            for name in ["hat", "cape", "amulet", "weapon"] {
                slot(ui, art, p, d, actions, name);
            }
        });
        portrait(ui, art, state, p, portrait_width);
        ui.vertical(|ui| {
            ui.set_width(58.);
            for name in ["ring1", "ring2", "belt", "boots"] {
                slot(ui, art, p, d, actions, name);
            }
        });
    });
    ui.add_space(8.);
    let stats = p.stats(d);
    egui::Frame::group(ui.style())
        .inner_margin(12.)
        .show(ui, |ui| {
            egui::Grid::new("character-stats")
                .num_columns(2)
                .min_col_width(130.)
                .spacing([10., 9.])
                .show(ui, |ui| {
                    for (label, value) in [
                        ("Vie maximale", format!("{:.0}", stats.hp)),
                        ("Puissance", format!("{:.0}", stats.power)),
                        (
                            "Réduction recharge",
                            format!("{:.0} %", stats.cooldown_reduction),
                        ),
                        (
                            "Bonus de déplacement",
                            format!("{:.0} %", stats.movement_speed),
                        ),
                        ("Kamas", p.kamas.to_string()),
                    ] {
                        ui.vertical(|ui| {
                            ui.small(label);
                            ui.colored_label(
                                color(),
                                egui::RichText::new(value).size(19.).strong(),
                            );
                        });
                        if label == "Puissance" || label == "Bonus de déplacement" {
                            ui.end_row();
                        }
                    }
                });
        });
    ui.small("Cliquez sur un équipement porté pour le retirer.");
}
fn bag(
    ui: &mut egui::Ui,
    art: &mut Art,
    state: &mut UiState,
    p: &Profile,
    d: &GameData,
    actions: &mut Actions,
) {
    let bag: Vec<_> = p
        .inventory
        .iter()
        .filter(|i| !p.equipment.values().any(|uid| uid == &i.uid))
        .filter(|i| state.filter.is_empty() || d.items[&i.item].set == state.filter)
        .collect();
    ui.horizontal(|ui| {
        ui.heading("Votre sac");
        ui.small(format!("{} objets", bag.len()));
    });
    ui.small("Double-cliquez sur un objet pour l'équiper.");
    egui::ComboBox::from_id_salt("filter")
        .selected_text(if state.filter.is_empty() {
            "Toutes les panoplies"
        } else {
            &d.sets[&state.filter].name
        })
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut state.filter, String::new(), "Toutes les panoplies");
            for (id, set) in &d.sets {
                ui.selectable_value(&mut state.filter, id.clone(), &set.name);
            }
        });
    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgb(27, 41, 44))
        .inner_margin(12.)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            if bag.is_empty() {
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), 225.),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        ui.add_space(65.);
                        ui.strong("Votre sac est vide");
                        ui.label("Les équipements portés restent visibles autour du personnage.");
                        ui.label("Retirez-en un pour le remettre dans le sac.");
                    },
                );
            } else {
                egui::ScrollArea::vertical()
                    .id_salt("bag")
                    .max_height(245.)
                    .min_scrolled_height(225.)
                    .show(ui, |ui| {
                        let cols = ((ui.available_width() / 61.).floor() as usize).max(1);
                        egui::Grid::new("bag-grid")
                            .spacing([5., 5.])
                            .show(ui, |ui| {
                                for (index, i) in bag.iter().enumerate() {
                                    let item = &d.items[&i.item];
                                    let selected = state.selected.as_ref() == Some(&i.uid);
                                    let response = egui::Frame::new()
                                        .fill(if selected {
                                            egui::Color32::from_rgb(42, 65, 53)
                                        } else {
                                            egui::Color32::from_rgb(17, 30, 33)
                                        })
                                        .corner_radius(6)
                                        .inner_margin(4.)
                                        .show(ui, |ui| {
                                            art.icon(ui, &item.icon, egui::vec2(46., 46.))
                                        })
                                        .inner
                                        .on_hover_ui(|ui| item_tooltip(ui, item, d));
                                    if response.clicked() {
                                        state.selected = Some(i.uid.clone());
                                    }
                                    if response.double_clicked() {
                                        actions.0.push(Action::Equip(i.uid.clone(), None));
                                    }
                                    if (index + 1) % cols == 0 {
                                        ui.end_row();
                                    }
                                }
                            });
                    });
            }
        });
    egui::Frame::group(ui.style())
        .inner_margin(12.)
        .show(ui, |ui| {
            ui.set_min_height(125.);
            ui.set_min_width(ui.available_width());
            if let Some(i) = state
                .selected
                .as_ref()
                .and_then(|uid| p.inventory.iter().find(|i| &i.uid == uid))
            {
                let item = &d.items[&i.item];
                ui.strong(&item.name);
                ui.label(stats_text(&item.stats));
                ui.colored_label(color(), &d.sets[&item.set].name);
                ui.horizontal(|ui| {
                    if item.category == "ring" {
                        for (slot, label) in [("ring1", "Anneau 1"), ("ring2", "Anneau 2")] {
                            if ui.button(label).clicked() {
                                actions
                                    .0
                                    .push(Action::Equip(i.uid.clone(), Some(slot.into())));
                            }
                        }
                    } else if ui.button("Équiper").clicked() {
                        actions.0.push(Action::Equip(i.uid.clone(), None));
                    }
                });
            } else {
                ui.strong("Un équipement à examiner ?");
                ui.label("Sélectionnez un objet pour consulter ses bonus et l'équiper.");
            }
        });
    let (sold, value) = p.sale_plan(d);
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{if ui.add_enabled(!sold.is_empty(),egui::Button::new(format!("Vendre doublons · +{value} kamas"))).on_hover_text("Sur tout le sac : protège les objets portés et conserve un exemplaire libre des autres objets.").clicked(){actions.0.push(Action::Sell);}});
}
pub(super) fn inventory(
    ctx: &egui::Context,
    art: &mut Art,
    state: &mut UiState,
    p: &Profile,
    d: &GameData,
    _s: &Session,
    actions: &mut Actions,
) {
    let mut open = state.inventory;
    egui::Window::new("Inventaire et équipement")
        .id(egui::Id::new("inventory"))
        .open(&mut open)
        .collapsible(false)
        .resizable(true)
        .vscroll(true)
        .default_size([1080., 690.])
        .max_size(ctx.content_rect().size() - egui::vec2(30., 120.))
        .anchor(egui::Align2::CENTER_CENTER, [0., -35.])
        .show(ctx, |ui| {
            ui.colored_label(color(), format!("VOTRE PERSONNAGE · NIVEAU {}", p.level));
            ui.add_space(12.);
            let width = (ui.available_width() * 0.34).clamp(300., 360.);
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(width, 0.),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| sheet(ui, art, state, p, d, actions),
                );
                ui.add_space(14.);
                ui.vertical(|ui| bag(ui, art, state, p, d, actions));
            });
            egui::ScrollArea::vertical()
                .id_salt("set-bonuses")
                .max_height(100.)
                .show(ui, |ui| {
                    for (id, pieces) in p.set_counts(d) {
                        if let Some(set) = d.sets.get(&id) {
                            egui::Frame::group(ui.style()).show(ui, |ui| {
                                ui.colored_label(
                                    color(),
                                    format!("{} · {} pièces distinctes", set.name, pieces.len()),
                                );
                                for (n, stats) in &set.thresholds {
                                    ui.small(format!(
                                        "{} {n} pièces : {}",
                                        if *n <= pieces.len() as u32 {
                                            "Actif"
                                        } else {
                                            "À obtenir"
                                        },
                                        stats_text(stats).replace('\n', " · ")
                                    ));
                                }
                            });
                        }
                    }
                });
            ui.small("Simulation en pause");
        });
    state.inventory = open;
}
