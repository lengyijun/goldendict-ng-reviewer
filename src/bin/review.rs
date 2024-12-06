use cursive::traits::*;
use cursive::views::Dialog;
use cursive::Cursive;
use cursive::CursiveExt;

fn main() {
    let mut siv = Cursive::default();

    // Create a linear layout to arrange buttons horizontally at the bottom
    // let button_layout = LinearLayout::horizontal()
    //     .child(easy_btn)
    //     .child(good_btn)
    //     .child(hard_btn)
    //     .child(again_btn);

    // Place the buttons at the bottom of the screen
    siv.add_fullscreen_layer(
        Dialog::new()
            .title("Select Difficulty")
            .button("Show answer", |s| {
                s.add_layer(Dialog::info("Good selected"));
                s.call_on_name("ocean", |view: &mut Dialog| {
                    view.set_title("132");
                    view.clear_buttons();
                    view.add_button("Easy", |s| {
                        s.add_layer(Dialog::info("Easy selected"));
                    });
                    view.add_button("Good", |s| {
                        s.add_layer(Dialog::info("Good selected"));
                    });
                    view.add_button("Hard", |s| {
                        s.add_layer(Dialog::info("Hard selected"));
                    });
                    view.add_button("Again", |s| {
                        s.add_layer(Dialog::info("Again selected"));
                    });
                });
            })
            .with_name("ocean"), // .content(button_layout)
                                 // .button("Quit", |s| s.quit()),
                                 // .padding(Margins::lrtb(10, 10, 0, 35))
    );

    siv.run();
}
