<script setup lang="ts">
import { onMounted, onUnmounted, ref, useTemplateRef } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Button from "primevue/button";
import InputText from "primevue/inputtext";
import * as echarts from "echarts/core";
import { GridComponent } from "echarts/components";
import { LineChart } from "echarts/charts";
import { UniversalTransition } from "echarts/features";
import { CanvasRenderer } from "echarts/renderers";
echarts.use([GridComponent, LineChart, CanvasRenderer, UniversalTransition]);

const greetMsg = ref("");
const name = ref("");
const chart = useTemplateRef("chart");

onMounted(() => {
    const chartInstance = echarts.init(chart.value);
    const option = {
        xAxis: {
            type: "category",
            data: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
        },
        yAxis: {
            type: "value",
        },
        series: [
            {
                data: [150, 230, 224, 218, 135, 147, 260],
                type: "line",
            },
        ],
    };
    chartInstance.setOption(option);
    onUnmounted(() => {
        echarts.dispose(chartInstance);
    });
});

async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg.value = await invoke("greet", { name: name.value });
}
</script>

<template>
    <main class="container">
        <h1>Welcome to Tauri + Vue</h1>

        <form class="row" @submit.prevent="greet">
            <InputText
                id="greet-input"
                v-model="name"
                placeholder="Enter a name..."
            />
            <Button type="submit">Greet</Button>
        </form>
        <p>{{ greetMsg }}</p>

        <div ref="chart" style="width: 100%; height: 400px"></div>
    </main>
</template>

<style scoped>
.container {
    margin: 0;
    padding-top: 10vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    text-align: center;
}

.row {
    display: flex;
    justify-content: center;
}
</style>
