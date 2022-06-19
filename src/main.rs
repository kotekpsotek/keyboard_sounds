use std::path::Path;
use std::time::Instant;
use std::fs;
use clap::{ Command, Arg };
use soloud::*;
use serde::{ Serialize, Deserialize };
use serde_json;
use device_query::{ DeviceState, DeviceQuery, Keycode };

const FOLDER_WITH_SONGS: &str = "./songs";
const BINDINGS_FILE_LOCALISATION: &str = "./bindings/bindings.json";

#[allow(dead_code)]
fn soloud_play_song(path_to_file: Option<&Path>) {
    // Import soloud engine bindings for Rustlang
    // TODO: If some song is already plaing then stop this song and play new

    let mut sl = Soloud::default().unwrap();

    // Load file to play into soloud engine
    let mut wav = audio::Wav::default();
    match path_to_file {
        Some(path) => {
            wav.load(path).unwrap();
        },
        None => wav.load(Path::new("songs/test_file.mp3")).unwrap()
    };
    

    // Call play sound
    sl.play(&wav);

    // Wait for inicjalize play sound and keep not stop play sopng until song end (this is default behaviour of solod)
    let time_elapser = Instant::now();
    let _song_time_milis = wav.length() * 1000.0;
    while sl.voice_count() > 0 {
        let _time_elapsed = time_elapser.elapsed().as_millis() as f64;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}


struct App;
impl App {
    fn run_app() {
        // Run application in conviniently way by call this assigned function
        let device_state = DeviceState::new();
        loop {
            // Get only one pressed key
            let pressed_keys = device_state.get_keys() as Vec<Keycode>;
            let determine_one_pressed_key = if pressed_keys.len() > 0 {
                pressed_keys[0].to_string()
            } else {
                String::new()
            };

            // Get Binding object for clicked key and play song from this object or do nothing when None is recived
            let get_stored_binding_or_none = BindingsSuite::check_exists_return_data(determine_one_pressed_key);
            match get_stored_binding_or_none {
                Some(binding_obj) => {
                    binding_obj.play_song_for_binding();
                },
                None => ()
            };
        };
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct BindingsSuite {
    bindings: Vec<Binding>
}

impl BindingsSuite {
    // Get all bindings from file by using one method for this pourpose
    fn file_get_all_bindings() -> Self {
        let path_to_bindings = Path::new(BINDINGS_FILE_LOCALISATION);

        // Error messages
        let couldnt_format_file_with_bindings = format!("File with bindings has got incorrect format!!!");
        let couldnt_read_bindings_file = "Couldn't read content file with bindings!";

        // Get All bindings from file
        let file_with_bindings_content = fs::read_to_string(path_to_bindings).expect(couldnt_read_bindings_file);
        let all_bindings_object = serde_json::from_str::<BindingsSuite>(&file_with_bindings_content).expect(&couldnt_format_file_with_bindings);

        // Return all bindings getted from file if exists or struct with empty bindings set
        all_bindings_object
    }

    // Return Binding object nested in "Option" enum which store keyboard key which has been tailored to "key_name" param or return "None"
    fn check_exists_return_data(key_name: String) -> Option<Binding> {
        // Get all bindings from file and return first binding object which key "key" store the same key as "key_name" added in function param
        let all_bindings = Self::file_get_all_bindings().bindings;
        let get_specific_binding = all_bindings.iter().find(|binding| binding.key.to_lowercase() == key_name.to_lowercase())?.to_owned();

        Some(get_specific_binding)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Binding {
    key: String,
    song_name: String
}

impl Binding {
    fn save(&self) -> bool {
        // Get Struct with prior saved bindings or with empty bindings set
        let path_to_bindings = Path::new(BINDINGS_FILE_LOCALISATION);
        let mut all_bindings_format = if path_to_bindings.exists() {
            // Get all bindings from file
            let mut all_bindings_object = BindingsSuite::file_get_all_bindings();

            // When any binding is on binding file then remove only one binding which is equal to new setting binding (only when the same binding has been found) in purpose to replace old key binding to new binding (only one will be replacing because .position() iterator method gets only first match and stop iteration over remaining elements)
            if all_bindings_object.bindings.len() > 0 {
                let search_same_binding = all_bindings_object.bindings.iter().position(|binding_from_file| binding_from_file.key == self.key);
                if let Some(index) = search_same_binding {
                    all_bindings_object.bindings.remove(index);
                }
            }

            // Return bindings instance
            all_bindings_object
        }
        else {
            BindingsSuite {
                bindings: Vec::new()
            }
        };

        // Add to already existing bindings this new bindings unstance
        all_bindings_format.bindings.push(self.clone());

        // Convert struct with all existing bindings and appended new binding to json format and save it into file with all bindings (in localisation to which refer this varaiable "BINDINGS_FILE_LOCALISATION") 
        let bindings_json_format_agian = serde_json::to_string(&all_bindings_format).expect(r#"From some reason couldn't convert again all bindings to "json" format"#);
        let _ = fs::write(path_to_bindings, bindings_json_format_agian).expect("Couldn't save new bindings to file with all already existing bindings");

        // Allways return true when error is not found
        true
    }

    // Play song this is endpoint of "Binding" struct life
    fn play_song_for_binding(self) {
        let song_file = self.song_name;

        // Check whether song exists and play it
        let path_to_song = Path::new(FOLDER_WITH_SONGS).join(&song_file);

        if path_to_song.exists() {
            let path_with_song = path_to_song.as_path();
            soloud_play_song(Some(path_with_song));
        }
        else {
            eprintln!(r#"Coludn't play this song "{song}" because this song doesn't exists in {library}"#, song = song_file, library = FOLDER_WITH_SONGS.split("/").collect::<Vec<&str>>()[1]);
        };
    }
}

fn main() {
    // play_using_codecs_and_stream();
    let app = Command::new("keybaords_sounds")
        .author("SiematuMichael@protonmail.com")
        .version("1.0")
        .about("Play sounds after click on keyboard key to specific device")
        .subcommand(Command::new("add-binding")
            .about("Add new binding and sound binding for keyboard single key")
            .args([
                Arg::new("keyboard key letter")
                    .help("required for restup binding. Only lower cases letters are supported and only one letter located on start will be recognized")
                    .takes_value(true)
                    .required(true),
                Arg::new("song file localisation")
                    .help("required for add play song efect for setted binding. Path to file can be relative or absolute")
                    .takes_value(true)
                    .required(true),
            ])
        )
        .subcommand(
            Command::new("run")
                .about("This command run the application and this is equavilent to inicjalize raw file")
        )
        .get_matches();
    
    // Handle specific subcommands commands
    if let Some(subcommand_data) = app.subcommand_matches("add-binding") {
        // Handle Command to add new song bindings
        // All arguments are required so additional check isn't required
        let keyboard_key_arg = subcommand_data.value_of("keyboard key letter").unwrap();
        let song_localisation_arg = subcommand_data.value_of("song file localisation").unwrap();

        // Convert Values to appropriately formats and types
        let keyboard_key = &keyboard_key_arg.to_lowercase()[0..1];
        let song_localisation = Path::new(song_localisation_arg);

        if song_localisation.exists() {
            // Copy file content to new file with the same name and same format
            let file_name = song_localisation.file_name().unwrap().to_str().unwrap();
            match fs::copy(song_localisation, format!("./songs/{}", file_name)) {
                Ok(_) => {
                    // Action save binding into file with bindings
                    let new_binding_instance = Binding { key: keyboard_key.to_string(), song_name: file_name.to_string() };
                    new_binding_instance.save();
                    println!("Keyboard binding has been save. Use this commnad again to with same <keyboard key letter> argument to change binding song!!!")
                },
                Err(_) => {
                    println!(r##"Couldn't create new file {} in folder "{}" which contains all bindings songs"##, file_name, FOLDER_WITH_SONGS)
                }
            };
        }
        else {
            println!("Added localisation to song file is incorrect!!!")
        };
    }
    else if let Some(_subcommand_data) = app.subcommand_matches("run") {
        // Run application after call "run" command
        App::run_app();
    }
    else {
        // Run aplication when no-one command/args has been recived from cli
        App::run_app();
    };
}

#[cfg(test)]
mod test {
    use crate::{ soloud_play_song, Binding, BindingsSuite };

    #[test]
    fn play_song_basic() {
        soloud_play_song(None)
    }

    #[test]
    fn save_new_binding_in_bindings_file() {
        let new_binding_instance = Binding {
            key: "test".to_string(),
            song_name: "test_song.mp3".to_string()
        };
        
        let save_action = new_binding_instance.save();
        
        // Print this or test result is error
        if save_action {
            println!("Success all bindings has been save in binding file!!!");
        }
    }

    #[test]
    fn get_specific_binding() {
        let binding = BindingsSuite::check_exists_return_data("test".to_string()).expect("Couldn't get this binding!!!");
        println!("Binding object: {:#?}", binding);
    }
}
