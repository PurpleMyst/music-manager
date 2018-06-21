#[macro_use]
extern crate kiss_ui;

#[macro_use]
extern crate duct;

use ::kiss_ui::prelude::*;

use ::kiss_ui::button::Button;
use ::kiss_ui::callback::{Callback, CallbackStatus};
use ::kiss_ui::container::Grid;
use ::kiss_ui::progress::ProgressBar;
use ::kiss_ui::text::TextBox;
use ::kiss_ui::timer::Timer;

struct MyCallback<Args, F: Fn(Args) -> ()>(F, ::std::marker::PhantomData<Args>);

impl<Args: 'static, F: Fn(Args) -> () + 'static> MyCallback<Args, F> {
    fn new(f: F) -> Self {
        Self {
            0: f,
            1: ::std::marker::PhantomData,
        }
    }
}

impl<Args: 'static, F: Fn(Args) -> () + 'static> Callback<Args> for MyCallback<Args, F> {
    fn on_callback(&mut self, args: Args) -> CallbackStatus {
        (self.0)(args);
        CallbackStatus::Default
    }
}

fn download<V: Into<Vec<u8>>>(progress_bar: ProgressBar, urls: V) {
    // TODO: Show a dialog in many places here.

    let maybe_handle: Result<duct::Handle, _> = cmd!(
        "youtube-dl",
        "--embed-thumbnail",
        "--add-metadata",
        "--extract-audio",
        "--ignore-errors",
        "--audio-format", "best",
        "-a", "-"
    ).input(urls)
        .dir("./download")
        .start();

    match maybe_handle {
        Ok(handle) => {
            progress_bar.show();

            Timer::new()
                .set_interval(1 * 1000)
                .set_on_interval(MyCallback::new(move |timer: Timer| {
                    match handle.try_wait() {
                        Ok(Some(output)) => {
                            println!("output: {:?}", output);
                            progress_bar.hide();
                            timer.destroy();
                        }

                        Ok(None) => {}

                        Err(_) => {
                            unimplemented!("error handling");
                        }
                    }
                }))
                .start();
        }
        Err(_) => unimplemented!("error handling"),
    }
}

fn main() {
    // I'm 99% sure they switched up Horizontal and Vertical on Grid.
    kiss_ui::show_gui(|| {
        let text_box = TextBox::new();
        text_box.set_multiline(true);
        text_box.set_visible_columns(80);
        text_box.set_visible_lines(10);

        let download_button = Button::new();
        download_button.set_label("Download");

        let progress_bar = ProgressBar::new();
        progress_bar.set_indefinite(true);
        progress_bar.set_orientation(Orientation::Horizontal);
        progress_bar.hide();

        {
            let text_box = text_box.clone();
            let progress_bar = progress_bar.clone();
            download_button.set_onclick(MyCallback::new(move |_| {
                download(progress_bar.clone(), text_box.get_text().to_string());
            }));
        }

        let move_button = Button::new();
        move_button.set_label("Move To");
        move_button.set_onclick(MyCallback::new(|_| {
            unimplemented!("clicked move button");
        }));

        let mut small_grid = Grid::new(children![download_button, move_button, progress_bar]);
        small_grid.set_orientation(Orientation::Vertical);
        small_grid.set_ndiv(5);

        let big_grid = Grid::new(children![text_box, small_grid]);

        let dialog = Dialog::new(big_grid);
        dialog.set_title("Music Manager");
        dialog
    });
}
