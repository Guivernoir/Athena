import { invoke } from "@tauri-apps/api/core";

const hook = {
    'receive_input': async (input) => {
        return await invoke('receive_input', {input});
    },
    'receive_mode': async (mode) => {
        return await invoke('receive_mode', {mode});
    },
    'receive_proficiency': async (proficiency) => {
        return await invoke('receive_proficiency', {proficiency});
    },
    'receive_personality': async (personality) => {
        return await invoke('receive_personality', {personality});
    },
    'send_output': async (output) => {
        return await invoke('send_output', {output});
    }
}

export default hook;