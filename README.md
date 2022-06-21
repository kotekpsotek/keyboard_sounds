# **Keyboard sounds**

## **What is this**:
Simple program for bind voices like songs to any numeric or alphabetic key of your keyboard. Program works on CLI so for launch program, add new binding or replace existing you should use specific command for this or add it to file with bindings manually.

## **How it works:**
1. You run .exe file just as a normal program or, user for this cargo if you woudn't like to launch .exe file. For launch file also occur command **"run"** which do the same as normal .exe file launch or you can run program using "cargo",
2. You can use command "add-binding <key_name> <song_localisation_file>" to assign songs to keyboards keys and change existing binding (one command to change existing binding and create new binding),
3. After click on key which has got assign song then you can hear playing song and after click on other key which has got assing song while earlier song persist then prior listening song will be replacing by new song and by this moment replacment song persist, or when you click on the same key then same song starts playing from the begining. Every repeat this action do the same,

## **Commands ad flags list:**
1. **--help / -h / help** - show help information with program description, information about author and list of all commands and flags
2. **add-binding <keyboard_key_letter> <localisation_to_music_file>** - commands for add/change binding. <keyboard_key_letter> - value putted for this argument must be single letter and for now this can be also capital letter or lower case letter becase program for this moment couldn't recognize size of letter so small maven the same as capital, <localisation_to_music_file> - under this argument must be substitute localisation of file with supported music format. Program will checking first whther localisation is correct and file exists then copy music file to "/songs" folder. To summerize: Program didn't change localisation of original file but copy content and name of original music file and create new file in "/songs" folder with the same name
3. **run** - additional command for run program. Run program in the same way as cargo and .exe file,

## **Supported file formats:**
<table>
    <tr>
        <th>Format</th>
        <th>Supported</th>
    </tr>
    <tr>
        <td>wav</td>
        <td>Yes</td>
    </tr>
    <tr>
        <td>mp3</td>
        <td>Yes</td>
    </tr>
    <tr>
        <td>ogg</td>
        <td>Yes</td>
    </tr>
    <tr>
        <td>mp4</td>
        <td>No</td>
    </tr>
    <tr>
        <td>aac</td>
        <td>No</td>
    </tr>
</table>