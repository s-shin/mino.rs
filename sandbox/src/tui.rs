use cursive::theme::BaseColor::*;
use cursive::theme::Color::*;
use cursive::theme::PaletteColor::*;
use cursive::views::{BoxView, Canvas};
use cursive::Cursive;

pub fn run() {
    let mut siv = Cursive::default();
    siv.add_global_callback('q', |s| s.quit());

    let mut theme = siv.current_theme().clone();
    theme.shadow = false;
    theme.palette[Background] = Dark(Black);
    siv.set_theme(theme);

    siv.add_layer(BoxView::with_fixed_size(
        (10, 20),
        Canvas::new(()).with_draw(|_, printer| {
            printer.print((1, 1), "hoge");
        }),
    ));

    siv.run();
}
