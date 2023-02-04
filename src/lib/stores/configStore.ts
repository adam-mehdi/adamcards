
import { writable } from "svelte/store";


type AppConfig = {
    is_dark_mode: boolean,
    is_textfield: boolean
};

const config: AppConfig = {
    is_dark_mode: false,
    is_textfield: false
};

const configStore = writable(
   config 
);

export default configStore;