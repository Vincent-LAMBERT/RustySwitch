const { invoke } = window.__TAURI__.core;

window.addEventListener("DOMContentLoaded", () => {
  setupApp();
});

let server;
async function setupApp () {
  await invoke("get_sound_server").then((x) => server = x);
  console.log(server)
  getAudioOutputs();
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
async function getAudioOutputs() {
  await invoke("get_audio_outputs", {soundServer: server}).then((sinks) => updateSelect("mainSelect", sinks));
  await invoke("get_audio_outputs", {soundServer: server}).then((sinks) => updateSelect("secondSelect", sinks));
}

function updateSelect(id, sinks) {
  let selectEl = document.getElementById(id);
  // The transmission from Rust to Javascript adds an unused null value
  let audioOutputs = sinks[0]
  let activeSink = sinks[1]
  for (var i = 0; i<audioOutputs.length; i++){
    var opt = document.createElement('option');
    opt.value = i;
    opt.innerHTML = audioOutputs[i];
    selectEl.appendChild(opt);
  }
}