use std::path::Path;
use std::time::Instant;
use std::fs;
use clap::{ Command, Arg };
use soloud::*;
use serde::{ Serialize, Deserialize };
use serde_json;

const FOLDER_WITH_SONGS: &str = "./songs";
const BINDINGS_FILE_LOCALISATION: &str = "./bindings/bindings.json";

#[allow(dead_code)]
fn soloud_play_song() {
    // Import soloud engine bindings for Rustlang

    let mut sl = Soloud::default().unwrap();

    // Load file to play into soloud engine
    let mut wav = audio::Wav::default();
    let path_to_file = Path::new("./songs/test_file.mp3");

    wav.load(path_to_file).unwrap();

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


#[derive(Debug, Serialize, Deserialize)]
struct BindingsSuite {
    bindings: Vec<Binding>
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
            // Error messages
            let couldnt_format_file_with_bindings = format!("File with bindings has got incorrect format!!!");
            let couldnt_read_bindings_file = "Couldn't read content file with bindings!";
            
            // Get All bindings from file
            let file_with_bindings_content = fs::read_to_string(path_to_bindings).expect(couldnt_read_bindings_file);
            let mut all_bindings_object = serde_json::from_str::<BindingsSuite>(&file_with_bindings_content).expect(&couldnt_format_file_with_bindings);

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
            ]))
        .get_matches();
    
    // TODO: Add function to save keyboard key binding with assinged song name (this function is already implemented in "Binding" methods struct)
    // Handle specific subcommands commands
    if let Some(subcommand_data) = app.subcommand_matches("add-binding") {
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
    };
}

#[cfg(test)]
mod test {
    use crate::{ soloud_play_song, Binding };

    #[test]
    fn play_song_basic() {
        soloud_play_song()
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
}
