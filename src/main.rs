mod math;

use eframe::Frame;
use egui::Ui;
use egui_graphs::{ForceAlgorithm, FruchtermanReingold, FruchtermanReingoldState, FruchtermanReingoldWithCenterGravity, FruchtermanReingoldWithCenterGravityState, LayoutForceDirected, LayoutHierarchical, LayoutState, LayoutStateHierarchical, SettingsNavigation};
use crate::math::*;

struct App {

    graph: egui_graphs::Graph,

}

impl eframe::App for App {

    fn ui(&mut self, ui: &mut Ui, frame: &mut Frame) {

        egui::CentralPanel::default().show(ui, |ui| {

            type L = LayoutForceDirected<FruchtermanReingoldWithCenterGravity>;
            type S = FruchtermanReingoldWithCenterGravityState;

            ui.add(&mut egui_graphs::GraphView::<_, _, _, _, _, _, S, L>::new(&mut self.graph)
                .with_navigations(&SettingsNavigation::new().with_zoom_and_pan_enabled(true).with_fit_to_screen_enabled(true )));

        });

    }
}

impl App {

    fn new(_: &eframe::CreationContext<'_>) -> Self {

        Self {

            graph: trivial_autocells::<2, 3>(true).adjacency_graph()
        }
    }
}

fn main() {

    trivial_autocells::<2,4>(false).write_edge_file("edges.csv".to_string());

    /*eframe::run_native(
        "AutoCells",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )*/
}