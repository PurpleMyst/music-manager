#[macro_use]
extern crate kiss_ui;

#[macro_use]
extern crate duct;

#[macro_use]
extern crate lazy_static;

use ::kiss_ui::prelude::*;

use ::kiss_ui::dialog::AlertPopupBuilder;
use ::kiss_ui::button::Button;
use ::kiss_ui::callback::{Callback, CallbackStatus};
use ::kiss_ui::container::{Horizontal, Vertical};
use ::kiss_ui::progress::ProgressBar;
use ::kiss_ui::text::{TextBox};
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

fn show_error_dialog(e: impl ::std::fmt::Display) {
    AlertPopupBuilder::new("There's been an error",
                           format!("{}", e),
                           "Ok").popup();
}

fn download<V: Into<Vec<u8>>>(progress_bar: ProgressBar, urls: V) {
    // TODO: Show a dialog in many places here.
    use ::std::sync::RwLock;

    lazy_static! {
        static ref BUSY: RwLock<bool> = RwLock::new(false);
    }

    if *BUSY.read().unwrap() {
        return;
    } else {
        *BUSY.write().unwrap() = true;
    }

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
                        Ok(Some(_)) => {
                            progress_bar.hide();
                            timer.destroy();
                            *BUSY.write().unwrap() = false;
                        }

                        Ok(None) => {}

                        Err(e) => {
                            show_error_dialog(e);
                            progress_bar.hide();
                            timer.destroy();
                            *BUSY.write().unwrap() = false;
                        }
                    }
                }))
                .start();
        }
        Err(e) => show_error_dialog(e),
    }
}

fn main() {
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

        let small_grid = Horizontal::new(children![download_button, move_button, progress_bar]);

        let big_grid = Vertical::new(children![text_box, small_grid]);

        let dialog = Dialog::new(big_grid);
        dialog.set_title("Music Manager");
        dialog
    });
}
