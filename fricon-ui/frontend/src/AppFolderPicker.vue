<script setup lang="ts">
const folderName = defineModel<string>();
import { InputText, InputGroup, Button } from "primevue";
import { open } from "@tauri-apps/plugin-dialog";

async function selectFolder() {
  const folder = await open({
    directory: true,
    multiple: false,
  });
  if (folder) {
    folderName.value = folder;
  }
}

function inputChanged() {
  console.log("Input changed");
}
</script>
<template>
  <InputGroup>
    <InputText
      v-model="folderName"
      @input="() => {}"
      placeholder="Enter folder name"
      @keyup.enter="inputChanged"
      @blur="inputChanged"
    />
    <Button label="Select Folder" @click="selectFolder" />
  </InputGroup>
</template>
