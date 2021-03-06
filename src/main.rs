use std::path::Path;
use std::time::{ Instant, Duration };
use std::fs;
use clap::{ Command, Arg };
use soloud::*;
use serde::{ Serialize, Deserialize };
use serde_json;
use device_query::{ DeviceState, DeviceQuery, Keycode };
use std::thread::{ spawn, sleep };

const FOLDER_WITH_SONGS: &str = "./songs";
const BINDINGS_FILE_LOCALISATION: &str = "./bindings/bindings.json";
const SONG_STATE_FILE: &str = "./state.txt";
const CHECK_SONG_PLAYING_STATE_DELAY_MS: u64 = 100;

#[allow(dead_code)]
fn soloud_play_song(path_to_file: Option<&Path>, song_file: String) {
    // This function use Soloud engine bindings for Rustlang. Soloud is C++ sound engine for games
    // Error which occurred description: Error occured when already song is playing and inicjalization of new song playing is called (this is feebly description of Soloud C++ sound engine error in bindings to)
    sleep(Duration::from_millis(100)); // Delay for give time for Song State system to apply appropriate changes (this causes that you don't get errors when you spam same key with hight frequency) FIXME: this depends on your device speed (cpu, disk read/save speed etc... for file system actions (in this case save/delete/read files)) change if you have a little bit worst machine

    // Create new instance of Soloud only when prior has been removed using RAII (Rust ownership system) because loop blocking I/O loop and in below and more speciffiically in "while" loop placed below is implemented system to stop playing song for give "place" to play new song across break blocking loop (loop which block program I/O loop)
    // Short conclusion: one instance Soloud per device in one time else you can get errors while program working
    if SongState::get_state() == 1 {
        let sl = Soloud::default().unwrap();

        // Load file to play into soloud engine
        let mut wav = audio::Wav::default();

        fn play_default_song_pleasant_for_ear(wav: &mut Wav, song_file: String) {
            eprintln!(r#"Coludn't play this song "{song}" because this song doesn't exists in {library}"#, song = song_file, library = FOLDER_WITH_SONGS.split("/").collect::<Vec<&str>>()[1]);
            println!("Default song was launched!!!");
            wav.load(Path::new("songs/test_file.mp3")).unwrap()
        }

        match path_to_file {
            Some(path) => {
                match path.exists() {
                    true => wav.load(path).unwrap(),
                    false => play_default_song_pleasant_for_ear(&mut wav, song_file)
                }
            },
            None => play_default_song_pleasant_for_ear(&mut wav, song_file)
        };
    
        // Call play sound
        sl.play(&wav);
    
        // Wait for inicjalize play sound and keep not stop play sopng until song end (this is default behaviour of solod)
        let time_elapser = Instant::now();
        let song_duration_ms = wav.length() * 1000.0;
        while sl.voice_count() > 0 {
            let time_elapsed = time_elapser.elapsed().as_millis() as f64;
    
            // Stop blocking program I/O loop by surrounded loop by free up space to create new Soloud instance and play new song
            if SongState::get_state() > 1 /* && time_elapsed >= 1000 as f64 */ {
                // When demand to play new song has been recived across capture song state change
                sl.stop_all();
                SongState::delete_state();
                break;
            }
            else if time_elapsed >= song_duration_ms {
                // When time expired
                SongState::delete_state();
                break;
            };
    
            sleep(std::time::Duration::from_millis(CHECK_SONG_PLAYING_STATE_DELAY_MS));
        };
    }
    else {
        // Recurency here
        SongState::rcreate_new_state();
        soloud_play_song(path_to_file, song_file);
    };

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

            // println!("{}", determine_one_pressed_key);
            // Get Binding object for clicked key and play song from this object or do nothing when None is recived
            let get_stored_binding_or_none = BindingsSuite::check_exists_return_data(determine_one_pressed_key);
            match get_stored_binding_or_none {
                Some(binding_obj) => {
                    // println!("Function to play song called!!!");

                    // Update or expand song play state
                    SongState::save_update_state();

                    // play song in loop and another thread
                    spawn(|| {
                        binding_obj.play_song_for_binding();
                    });
                    
                    // constrain call same song for key by freeze thread for some miliseconds
                    sleep(Duration::from_millis(500));
                },
                None => ()
            };
        };
    }
}

struct SongState;
impl SongState {
    #[allow(dead_code)]
    fn rcreate_new_state() {
        // Delete old state
        Self::delete_state();

        // Create new state file
        Self::save_update_state();
    }
    
    fn save_update_state() {
        let st_val = Self::get_state();
        let new_file_content = st_val + 1;
        let pf = Path::new(SONG_STATE_FILE);
        fs::write(pf, new_file_content.to_string()).unwrap();

    }

    fn get_state() -> u8 {
        let path_to_file = Path::new(SONG_STATE_FILE);
        let file_content = if path_to_file.exists() {
            let expr = fs::read_to_string(path_to_file);
            match expr {
                Ok(v) => {
                    let parse_res = v.parse::<u8>();
                    match parse_res {
                        Ok(v) => v,
                        Err(_) => 0
                    }
                },
                Err(_) => 0
            }
        }
        else {
            0
        };

        file_content
    }

    fn delete_state() {
        let pf = Path::new(SONG_STATE_FILE);
        
        if pf.exists() {
            fs::remove_file(pf).unwrap();
        }
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

        // Inicjalize source play sound function
        let path_with_song = path_to_song.as_path();
        soloud_play_song(Some(path_with_song), song_file);
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
    use crate::{ soloud_play_song, Binding, BindingsSuite, SongState };

    #[test]
    fn play_song_basic() {
        soloud_play_song(None, "./songs/test_file.mp3".to_string())
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

    #[test]
    fn music_state_1() {
        println!("{}", SongState::get_state())
    }
}
