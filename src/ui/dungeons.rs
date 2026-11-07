use super::*;
pub(super) fn dungeons(
    ctx: &egui::Context,
    art: &mut Art,
    state: &mut UiState,
    d: &GameData,
    actions: &mut Actions,
) {
    let mut open = state.dungeons;
    egui::Window::new("Choisir un donjon").open(&mut open).collapsible(false).resizable(true).default_size([1080.,650.]).max_size(ctx.content_rect().size()-egui::vec2(25.,120.)).anchor(egui::Align2::CENTER_CENTER,[0.,-35.]).show(ctx,|ui|{ui.label("Entrée libre, sans clef ni prérequis. Tous les gains sont conservés en cas de défaite.");let mut entries:Vec<_>=d.dungeons.values().collect();entries.sort_by_key(|d|d.tier);egui::ScrollArea::vertical().show(ui,|ui|{let count=if ui.available_width()>800.{3}else if ui.available_width()>500.{2}else{1};for row in entries.chunks(count){ui.columns(count,|columns|{for(ui,dungeon)in columns.iter_mut().zip(row){egui::Frame::group(ui.style()).inner_margin(12.).show(ui,|ui|{ui.strong(&dungeon.name);if let Some(boss)=dungeon.boss_wave.iter().map(|id|&d.monsters[id]).find(|m|m.boss){let(rect,response)=ui.allocate_exact_size(egui::vec2(ui.available_width(),125.),egui::Sense::click());let animated=(response.hovered()||response.has_focus())&&!state.reduced_motion;let id=format!("preview.mob.{}.{}.R",boss.sprite,if animated{"attack"}else{"idle"});art.in_rect(ui.painter(),&id,ctx.input(|i|i.time)as f32,rect,animated);ui.label(&boss.name);ui.horizontal_wrapped(|ui|{for drop in boss.loot.iter().chain(boss.bonus_loot.iter().flat_map(|bonus|bonus.table.iter())){let item=&d.items[&drop.item];art.icon(ui,&item.icon,egui::vec2(29.,29.)).on_hover_ui(|ui|item_tooltip(ui,item,d));}});}
 if ui.add_sized([ui.available_width(),32.],egui::Button::new("Partir à l’aventure").fill(egui::Color32::from_rgb(120,78,33))).clicked(){actions.0.push(Action::Enter(dungeon.id.clone(),false));state.dungeons=false;}if ui.add_sized([ui.available_width(),30.],egui::Button::new("Défier en Extrême")).clicked(){actions.0.push(Action::Enter(dungeon.id.clone(),true));state.dungeons=false;}});}});ui.add_space(8.);}});});
    state.dungeons &= open;
}
